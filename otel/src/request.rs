use anyhow::Context as _;
use phper::{
    eg,
    ini::ini_get,
    sg,
    sys,
    arrays::{IterKey, ZArr},
    values::ZVal,
};
use once_cell::sync::Lazy;
use std::{
    cell::RefCell,
    collections::HashMap,
    env,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    sync::Mutex,
};
use opentelemetry::{
    Context,
    InstrumentationScope,
    KeyValue,
    global,
    trace::{SpanKind, Tracer, TraceContextExt, TracerProvider},
};
use opentelemetry_semantic_conventions as SemConv;
use crate::{
    config,
    context::storage,
    error::php_error_to_attributes,
    trace::{local_root_span, tracer_provider},
    util::{get_sapi_module_name},
};

thread_local! {
    static OTEL_REQUEST_GUARD: RefCell<Option<opentelemetry::ContextGuard>> = RefCell::new(None);
    static OTEL_CONTEXT_ID: RefCell<Option<u64>> = RefCell::new(None);
}
//backup mutating environment variables for request duration
static ENV_BACKUP: Lazy<Mutex<HashMap<u32, HashMap<String, String>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn init_environment() {
    let dotenv_enabled = ini_get::<bool>(config::ini::OTEL_ENV_DOTENV_ENABLED);
    let env_from_server = ini_get::<bool>(config::ini::OTEL_ENV_SET_FROM_SERVER);

    if dotenv_enabled || env_from_server {
        backup_env();
    }
    if env_from_server {
        set_env_from_server();
    }
    if dotenv_enabled {
        process_dotenv();
    }
}

/// set OTEL_* variables from $_SERVER into process environment
/// This is useful for SAPI modules like Apache, where environment variables are set by SetEnv or
/// similar directives, and are not available in the process environment by default.
fn set_env_from_server() {
    let otel_vars = get_server_vars_with_prefix("OTEL_");
    for (k, v) in otel_vars {
        unsafe { std::env::set_var(&k, &v) };
        tracing::debug!("Set environment variable from $_SERVER: {}={}", k, v);
    }
}

/// Process the .env file found between SCRIPT_FILENAME and DOCUMENT_ROOT, setting select OTEL_*
/// variables found there into the process environment (OTEL_SERVICE_NAME,
/// OTEL_RESOURCE_ATTRIBUTES, OTEL_SDK_DISABLED).
fn process_dotenv() {
    if let Some(env_path) = find_dotenv() {
        tracing::debug!("Discovered .env path: {:?}", env_path);
        let mut service_name = None;
        let mut resource_attributes = None;
        let mut otel_disabled = None;
        if let Ok(iter) = dotenvy::from_path_iter(&env_path) {
            for item in iter.flatten() {
                match item.0.as_str() {
                    "OTEL_SERVICE_NAME" => service_name = Some(item.1),
                    "OTEL_RESOURCE_ATTRIBUTES" => resource_attributes = Some(item.1),
                    "OTEL_SDK_DISABLED" => otel_disabled = Some(item.1),
                    _ => {}
                }
            }
            //now we _might_ have service name and resource attributes
            let mut env = HashMap::new();
            if let Some(service_name) = service_name {
                env.insert("OTEL_SERVICE_NAME".to_string(), service_name);
            }
            if let Some(otel_disabled) = otel_disabled {
                env.insert("OTEL_SDK_DISABLED".to_string(), otel_disabled);
            }
            if let Some(resource_attributes) = resource_attributes {
                //merge with original env var, if it exists
                let mut merged = if let Some(existing) = std::env::var("OTEL_RESOURCE_ATTRIBUTES").ok() {
                    parse_resource_attributes(&existing)
                } else {
                    HashMap::new()
                };

                // Overwrite with values from dotenv
                for (k, v) in parse_resource_attributes(&resource_attributes) {
                    merged.insert(k, v);
                }

                // Serialize back to comma-separated key=value pairs
                let merged_str = {
                    let mut items: Vec<_> = merged.into_iter().collect();
                    items.sort_by(|a, b| a.0.cmp(&b.0));
                    items
                        .into_iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(",")
                };

                env.insert("OTEL_RESOURCE_ATTRIBUTES".to_string(), merged_str);
            }
            set_request_dotenv(env);
        }
    } else {
        tracing::warn!("No .env file found between SCRIPT_FILENAME and DOCUMENT_ROOT");
    }
}

/// Find the nearest .env file starting from the script's directory, finishing at DOCUMENT_ROOT.
/// If DOCUMENT_ROOT is not set, it will only look in the script's directory.
fn find_dotenv() -> Option<PathBuf> {
    let script_filename = get_server_var("SCRIPT_FILENAME")?;
    let script_dir = Path::new(&script_filename).parent()?;
    let env_in_dir = |dir: &Path| {
        let env_path = dir.join(".env");
        fs::metadata(&env_path).ok().map(|_| env_path)
    };

    match get_server_var("DOCUMENT_ROOT") {
        Some(document_root) => {
            if document_root.is_empty() {
                return env_in_dir(script_dir);
            }
            let docroot = match Path::new(&document_root).canonicalize() {
                Ok(path) => path,
                Err(err) => {
                    tracing::warn!("Failed to canonicalize DOCUMENT_ROOT '{}': {}", document_root, err);
                    return env_in_dir(script_dir);
                },
            };
            let mut current = match script_dir.canonicalize() {
                Ok(path) => path,
                Err(_) => return None,
            };
            loop {
                if let Some(env_path) = env_in_dir(&current) {
                    return Some(env_path);
                }
                if current == docroot {
                    break;
                }
                if !current.pop() {
                    break;
                }
            }
            None
        }
        None => env_in_dir(script_dir),
    }
}

/// Initialize the request handler, creating a root span if necessary.
pub fn init() {
    tracing::debug!("RINIT::initializing request handler");
    unsafe {
        //ensure $_SERVER is populated
        let mut server = "_SERVER".to_string();
        sys::zend_is_auto_global_str(server.as_mut_ptr().cast(), server.len());
    }
    let sapi = get_sapi_module_name();
    tracing::debug!("RINIT::sapi module name is: {}", sapi.clone());
    let mut span_name: Option<String> = None;
    if sapi == "cli" {
        let trace_cli = ini_get::<bool>(config::ini::OTEL_CLI_CREATE_ROOT_SPAN);
        if trace_cli {
            tracing::debug!("RINIT::tracing cli enabled by ini");
            span_name = Some("php:cli".to_string());
        } else {
            tracing::debug!("RINIT::not auto-creating root span...");
            return;
        }
    }
    let request_details = get_request_details();
    if span_name.is_none() {
        span_name = match &request_details.method {
            Some(method) => Some(format!("{}", method)),
            None => Some("<unknown>".to_string()),
        };
    }

    tracing::debug!("RINIT::otel request is being traced, name={}", span_name.clone().unwrap_or("unknown".to_string()));
    let tracer_provider = tracer_provider::get_tracer_provider();
    let scope = InstrumentationScope::builder("php:rinit").build();
    let tracer = tracer_provider.tracer_with_scope(scope);
    let span_builder = tracer.span_builder(span_name.unwrap_or("unknown".to_string()));
    let mut attributes = span_builder.attributes.clone().unwrap_or_default();
    attributes.push(KeyValue::new(SemConv::trace::URL_FULL, request_details.uri.unwrap_or_default()));
    attributes.push(KeyValue::new(SemConv::trace::HTTP_REQUEST_METHOD, request_details.method.unwrap_or_default()));
    //attributes.push(KeyValue::new(SemConv::trace::HTTP_REQUEST_BODY_SIZE, request_details.body_length.unwrap_or_default())); //experimental
    // TODO other HTTP attributes are experimental

    let mut span_builder = span_builder.clone().with_attributes(attributes);
    span_builder.span_kind = Some(SpanKind::Server);
    let parent_context = get_propagated_context();
    let is_local_root = !Context::current().span().span_context().is_valid();
    let span = tracer.build_with_context(span_builder, &parent_context);
    let ctx = Context::current_with_span(span);
    let context_id = storage::store_context_instance(Arc::new(ctx.clone()));
    OTEL_CONTEXT_ID.with(|cell| {
        *cell.borrow_mut() = Some(context_id);
    });
    if is_local_root {
        tracing::debug!("RINIT::is_local_root: {}", is_local_root);
        local_root_span::store_local_root_span(context_id);
    }
    //TODO use span::storeInContext logic
    let guard = ctx.attach();

    OTEL_REQUEST_GUARD.with(|slot| {
        *slot.borrow_mut() = Some(guard);
    });
    tracing::debug!("RINIT::request initialized");
}

/// Shutdown the request handler, closing the root span if it exists.
/// Restore the environment variables to their original state.
pub fn shutdown() {
    restore_env();
    let context_id = OTEL_CONTEXT_ID.with(|cell| cell.borrow_mut().take());
    let is_tracing = context_id.is_some();
    if is_tracing {
        let context_id = context_id.unwrap();
        let is_http_request = get_sapi_module_name() != "cli";
        tracing::debug!("RSHUTDOWN::auto-closing root span...");
        let ctx = storage::get_context_instance(context_id);
        if ctx.is_none() {
            tracing::warn!("RSHUTDOWN::no context found for id {}", context_id);
            return;
        }
        let ctx = ctx.unwrap();
        let span = ctx.span();
        if span.span_context().is_valid() {
            if is_http_request {
                let response_code = get_response_status_code();
                span.set_attribute(KeyValue::new(SemConv::trace::HTTP_RESPONSE_STATUS_CODE, response_code as i64));
                if response_code >= 500 {
                    let mut func = ZVal::from("error_get_last");
                    let mut args: Vec<ZVal> = Vec::new();
                    let error = ZVal::call(&mut func, &mut args).ok();
                    if let Some(error) = error {
                        if error.get_type_info().is_array() {
                            tracing::debug!("RSHUTDOWN::HTTP error detected: {:?}", error);
                            let attributes = php_error_to_attributes(&error);
                            span.add_event("exception", attributes);
                        }
                    }
                    // https://opentelemetry.io/docs/specs/semconv/http/http-spans/#status
                    span.set_status(opentelemetry::trace::Status::error(""));
                }
            }
            span.end();
            tracing::debug!("RSHUTDOWN::removing context: {}", context_id);
            drop(ctx);
            storage::maybe_remove_context_instance(context_id);
        }

        OTEL_REQUEST_GUARD.with(|slot| {
            *slot.borrow_mut() = None;
        });
        OTEL_CONTEXT_ID.with(|slot| {
            *slot.borrow_mut() = None;
        });
    } else {
        tracing::debug!("RSHUTDOWN::not auto-closing root span...");
    }
    //final check: there should be zero stored context
    let stored_context_ids = storage::get_context_ids();
    if !stored_context_ids.is_empty() {
        tracing::warn!("RSHUTDOWN::context still stored: {:?}", stored_context_ids);
    } else {
        tracing::debug!("RSHUTDOWN::CONTEXT_STORAGE is empty :)");
    }
    storage::clear_context_storage();
}

pub fn get_request_details() -> RequestDetails {
    unsafe {
        //depending in SAPI, request_info.request_uri may not be what we want (eg "index.php" instead of url)
        let request_info = sg!(request_info);
        let server = get_request_server();
        let uri = server
            .ok()
            .and_then(|server| server.get("REQUEST_URI"))
            .and_then(|zv| z_val_to_string(zv))
            // Fallback to request_info.request_uri if not found
            .or_else(|| {
                Some(request_info.request_uri)
                    .filter(|ptr| !ptr.is_null())
                    .map(|ptr| std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned())
            });

        RequestDetails {
            method: Some(request_info.request_method)
                .filter(|ptr| !ptr.is_null())
                .map(|ptr| std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()),
            uri,
            body_length: request_info.content_length as u64,
            content_type: Some(request_info.content_type)
                .filter(|ptr| !ptr.is_null())
                .map(|ptr| std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()),
        }
    }
}

#[allow(dead_code)]
pub struct RequestDetails {
    pub method: Option<String>,
    uri: Option<String>,
    body_length: u64,
    content_type: Option<String>,
}

// @see https://github.com/apache/skywalking-php/blob/v0.8.0/src/request.rs#L152
#[allow(static_mut_refs)]
fn get_request_server<'a>() -> anyhow::Result<&'a ZArr> {
    unsafe {
        // Ensure $_SERVER is initialized
        let mut server = "_SERVER".to_string();
        sys::zend_is_auto_global_str(server.as_mut_ptr().cast(), server.len());
        let symbol_table = ZArr::from_mut_ptr(&mut eg!(symbol_table));
        let carrier = symbol_table
            .get("_SERVER")
            .and_then(|carrier| carrier.as_z_arr())
            .context("$_SERVER is null")?;
        Ok(carrier)
    }
}

fn get_server_vars_with_prefix(prefix: &str) -> HashMap<String, String> {
    get_request_server()
        .ok()
        .map(|server| {
            server
                .iter()
                .filter_map(|(key, value)| {
                    if let IterKey::ZStr(zstr) = key {
                        if let Ok(key_str) = zstr.to_str() {
                            if key_str.starts_with(prefix) {
                                if let Some(value_str) = z_val_to_string(value) {
                                    return Some((key_str.to_string(), value_str));
                                }
                            }
                        }
                    }
                    None
                })
                .collect()
        })
        .unwrap_or_default()
}

fn get_server_var(key: &str) -> Option<String> {
    get_request_server()
        .ok()
        .and_then(|server| server.get(key))
        .and_then(|zv| z_val_to_string(zv))
}

fn extract_request_headers(server: &ZArr) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    for (key, value) in server.iter() {
        if let IterKey::ZStr(zstr) = key {
            if let Ok(key_str) = zstr.to_str() {
                if key_str.starts_with("HTTP_") {
                    if let Some(value_str) = value.as_z_str().and_then(|z| z.to_str().ok()) {
                        // Convert HTTP_ header names to standard format (e.g., HTTP_USER_AGENT -> User-Agent)
                        let header_name = key_str
                            .trim_start_matches("HTTP_")
                            .replace('_', "-")
                            .to_ascii_lowercase();

                        headers.insert(header_name, value_str.to_string());
                    }
                }
            }
        }
    }

    headers
}

fn z_val_to_string(zv: &ZVal) -> Option<String> {
    zv.as_z_str()
        .and_then(|zs| zs.to_str().ok())
        .map(|s| s.to_string())
}

fn get_response_status_code() -> i32 {
    unsafe { sg!(sapi_headers).http_response_code }
}

// Parse a comma-separated key=value string into a HashMap
fn parse_resource_attributes(s: &str) -> HashMap<String, String> {
    s.split(',')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) if !key.is_empty() && !value.is_empty() => Some((key.trim().to_string(), value.trim().to_string())),
                _ => None,
            }
        })
        .collect()
}

fn get_propagated_context() -> Context {
    let server = match get_request_server() {
        Ok(server) => server,
        Err(_) => return Context::current(),
    };

    // Extract headers from `$_SERVER`
    let headers = extract_request_headers(server);

    global::get_text_map_propagator(|prop| prop.extract(&headers))
}

//per-request .env support
pub fn set_request_dotenv(env: HashMap<String, String>) {
    for (k, v) in env {
        unsafe { std::env::set_var(&k, &v) };
        tracing::debug!("Set environment variable from .env {}={}", k, v);
    }
}

/// Check if OpenTelemetry is disabled for the current request (by env or .env)
pub fn is_disabled() -> bool {
    match std::env::var("OTEL_SDK_DISABLED") {
        Ok(val) => val == "true",
        Err(_) => false,
    }
}

fn backup_env() {
    tracing::debug!("Backing up OTEL_* environment variables");
    let env = env::vars()
        .filter(|(k, _)| k.starts_with("OTEL_"))
        .collect::<HashMap<_, _>>();
    let pid = std::process::id();
    ENV_BACKUP.lock().unwrap().insert(pid, env);
}

fn restore_env() {
    tracing::debug!("Restoring environment variables from backup");
    let pid = std::process::id();
    if let Some(backup) = ENV_BACKUP.lock().unwrap().remove(&pid) {
        // Remove any new env vars not in the backup
        for (k, _) in env::vars() {
            if !backup.contains_key(&k) && k.starts_with("OTEL_") {
                tracing::debug!("Removing added environment variable {}", k);
                unsafe { env::remove_var(&k) };
            }
        }
        // Restore all backed up vars
        for (k, v) in backup {
            unsafe { env::set_var(k, v) };
        }
    }
}