//! Per-request lifecycle: the RINIT and RSHUTDOWN halves of a PHP request.
//!
//! [`on_request_init`] and [`on_request_shutdown`] are the only entry points the module
//! wires up (see [`crate::module`]); everything else here exists to serve them. The two must
//! stay symmetric — a request runs in a worker process that will serve the next one, so
//! anything RINIT establishes (an active context, mutated environment, a root span) has to
//! be unwound in RSHUTDOWN or it leaks across the request boundary into an unrelated trace.
//!
//! The work splits into four concerns:
//!
//! - **Disabling.** `otel.disabled`, `.env`, and the excluded-URL list decide whether this
//!   request is traced at all. [`is_disabled`] is on the hot path — consulted once per
//!   auto-instrumented function — so its answer is memoized per request.
//! - **Environment.** A request may carry its own `.env`, which is applied over the process
//!   environment and restored at RSHUTDOWN from `ENV_BACKUP`. The SDK reads configuration
//!   from env vars, so this is how per-directory config reaches it at all.
//! - **The root span.** For a web request, RINIT opens a span from the propagated remote
//!   context, named for the HTTP method, and RSHUTDOWN finishes it with the response status.
//!   Both are skipped under CLI unless `otel.cli.enabled` is set.
//! - **`$_SERVER` reading.** Request attributes (method, uri, headers) and the incoming
//!   traceparent all come from `$_SERVER`, which is why RINIT has to force-populate it when
//!   `auto_globals_jit` would otherwise leave it empty.
//!
//! All request state lives in thread-locals rather than process statics, because a request
//! belongs to a thread: under a threaded SAPI, sharing them would let one request's root
//! span or env backup clobber another's.
use anyhow::Context as _;
use phper::{
    eg,
    ini::ini_get,
    pg,
    sg,
    sys,
    arrays::{IterKey, ZArr},
    values::ZVal,
};
use regex::Regex;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    env,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use opentelemetry::{
    global,
    Context,
    InstrumentationScope,
    KeyValue,
    trace::{SpanKind, Tracer, TraceContextExt, TracerProvider},
};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_semantic_conventions as SemConv;
use crate::{
    auto,
    config,
    context::storage,
    logging,
    logs::logger_provider,
    module,
    error::php_error_to_attributes,
    trace::{local_root_span, tracer_provider},
    util::{get_sapi_module_name},
};

thread_local! {
    static OTEL_REQUEST_GUARD: RefCell<Option<opentelemetry::ContextGuard>> = const { RefCell::new(None) };
    static OTEL_CONTEXT_ID: RefCell<Option<u64>> = const { RefCell::new(None) };
    /// Memoized result of `is_disabled()` for the current request. The answer cannot
    /// change within a request, but `is_disabled()` is on the hot path (it is consulted
    /// by every `get_tracer_provider`/`get_logger_provider` call, i.e. once per
    /// auto-instrumented function), and computing it reads `$_SERVER` and re-parses
    /// the excluded-URL list. Cleared in RSHUTDOWN.
    static DISABLED_FOR_REQUEST: Cell<Option<bool>> = const { Cell::new(None) };
    /// The excluded-URL list as it was last seen, with the regex compiled from it (`None`
    /// if that list compiled to nothing). Compiling is far more expensive than matching,
    /// and the list rarely changes, so hold onto it across requests.
    static EXCLUDED_URL_CACHE: RefCell<Option<(String, Option<Regex>)>> = const { RefCell::new(None) };
    /// The OTEL_* environment as it was before this request's `.env` was applied, so
    /// RSHUTDOWN can put it back.
    ///
    /// Thread-local, not keyed by pid: a request belongs to a thread, and under a threaded
    /// SAPI two concurrent requests in one process would otherwise share (and clobber) a
    /// single pid-keyed backup.
    static ENV_BACKUP: RefCell<Option<HashMap<String, String>>> = const { RefCell::new(None) };
}

/// RINIT handler.
pub fn on_request_init() {
    if module::is_disabled() {
        return;
    }
    jit_initialization();
    logging::init_once();
    tracing::debug!("OpenTelemetry::RINIT");
    init_environment();

    if is_disabled() {
        tracing::debug!("OpenTelemetry::RINIT: SDK disabled, skipping initialization");
        return;
    }

    tracer_provider::init_once();
    logger_provider::init_once();
    global::set_text_map_propagator(TraceContextPropagator::new());

    init();
}

/// RSHUTDOWN handler. Invoke request shutdown logic, call shutdown() on plugin manager.
pub fn on_request_shutdown() {
    // Before the disabled check, not after: the PHP classes are registered either way, so
    // `Context::with()` can store PHP values during a request the extension is disabled for.
    // Those are `ZVal`s -- PHP memory freed with the request -- so holding them in a
    // thread-local past this point is a use-after-free on the next request on a reused worker,
    // not a leak. Everything `shutdown()` cleans up below is plain Rust and can wait.
    storage::clear_context_values();
    if module::is_disabled() {
        return;
    }
    tracing::debug!("OpenTelemetry::RSHUTDOWN");
    shutdown();
    if let Some(plugin_manager) = auto::plugin_manager::get_global() {
        // A poisoned lock means an earlier panic; the manager is only mutated during MINIT,
        // so its contents are still sound to read. Skipping per-request plugin cleanup here
        // would leak this request's state into the next one.
        let pm = match plugin_manager.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        pm.request_shutdown();
    }
}

/// Check if OpenTelemetry is disabled for the current request (by env, .env or excluded URL)
///
/// The result is memoized for the duration of the request; see `DISABLED_FOR_REQUEST`.
pub fn is_disabled() -> bool {
    if let Some(cached) = DISABLED_FOR_REQUEST.get() {
        return cached;
    }
    let disabled = compute_is_disabled();
    DISABLED_FOR_REQUEST.set(Some(disabled));
    disabled
}

fn compute_is_disabled() -> bool {
    let request_details = get_request_details();

    // Check for excluded URL before creating root span
    if let Some(ref uri) = request_details.uri
        && is_excluded_url(uri) {
        tracing::debug!("RINIT::excluded URL matched '{}'", uri);
        return true;
    }
    match std::env::var("OTEL_SDK_DISABLED") {
        Ok(val) => val == "true",
        Err(_) => false,
    }
}

pub fn get_request_details() -> RequestDetails {
    unsafe {
        //depending in SAPI, request_info.request_uri may not be what we want (eg "index.php" instead of url)
        let request_info = sg!(request_info);
        let server = get_request_server();
        let uri = server
            .ok()
            .and_then(|server| server.get("REQUEST_URI"))
            .and_then(z_val_to_string)
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

fn init_environment() {
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
                let mut merged = if let Ok(existing) = std::env::var("OTEL_RESOURCE_ATTRIBUTES") {
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

/// Translate one comma-separated excluded-URL list into a regex that matches any of its
/// patterns in full.
///
/// `*` is a wildcard matching any run of characters; everything else is literal, so a
/// pattern containing regex metacharacters (`.`, `?`, `+`, bare `(`) matches those
/// characters rather than being interpreted — or failing to compile.
fn compile_excluded_urls(list: &str) -> Option<Regex> {
    let alternatives: Vec<String> = list
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|pattern| {
            pattern
                .split('*')
                .map(regex::escape)
                .collect::<Vec<_>>()
                .join(".*")
        })
        .collect();

    if alternatives.is_empty() {
        return None;
    }
    // Anchored: a pattern has to describe the whole URI, as the un-wildcarded case always
    // did (it compared for equality).
    let combined = format!("^(?:{})$", alternatives.join("|"));
    match Regex::new(&combined) {
        Ok(re) => Some(re),
        Err(err) => {
            // Escaping should make this unreachable; if it ever happens, excluding nothing
            // is the safer failure than excluding everything.
            tracing::error!("Failed to compile excluded URL patterns '{}': {}", list, err);
            None
        }
    }
}

/// Returns true if the given URI matches any pattern in the excluded URLs list.
/// Patterns support '*' as a wildcard (prefix/suffix/anywhere).
fn is_excluded_url(uri: &str) -> bool {
    let Ok(list) = std::env::var("OTEL_PHP_EXCLUDED_URLS") else {
        return false;
    };
    tracing::debug!("is_excluded_url: OTEL_PHP_EXCLUDED_URLS='{}'", list);

    // Keyed on the list itself rather than compiled once: the value comes from the
    // environment, which a `.env` file can change between requests in a reused worker.
    EXCLUDED_URL_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.as_ref().map(|(cached, _)| cached.as_str()) != Some(list.as_str()) {
            *cache = Some((list.clone(), compile_excluded_urls(&list)));
        }
        match cache.as_ref() {
            Some((_, Some(re))) => re.is_match(uri),
            _ => false,
        }
    })
}

/// Initialize the request handler, creating a root span if necessary.
fn init() {
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
            Some(method) => Some(method.to_string()),
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
        *cell.borrow_mut() = context_id;
    });
    if is_local_root {
        tracing::debug!("RINIT::is_local_root: {}", is_local_root);
        local_root_span::store_local_root_span(context_id);
    }
    let guard = ctx.attach();

    OTEL_REQUEST_GUARD.with(|slot| {
        *slot.borrow_mut() = Some(guard);
    });
    tracing::debug!("RINIT::request initialized");
}

/// Shutdown the request handler, closing the root span if it exists.
/// Restore the environment variables to their original state.
fn shutdown() {
    restore_env();
    close_root_span();

    // Request-scoped cleanup is unconditional: these thread-locals outlive the request
    // on a reused worker, so returning early on an unexpected path would leak guards
    // and context instances into subsequent requests.
    OTEL_REQUEST_GUARD.with(|slot| {
        *slot.borrow_mut() = None;
    });
    OTEL_CONTEXT_ID.with(|slot| {
        *slot.borrow_mut() = None;
    });
    DISABLED_FOR_REQUEST.set(None);

    //final check: there should be zero stored context
    let stored_context_ids = storage::get_context_ids();
    if !stored_context_ids.is_empty() {
        tracing::warn!("RSHUTDOWN::context still stored: {:?}", stored_context_ids);
    } else {
        tracing::debug!("RSHUTDOWN::CONTEXT_STORAGE is empty :)");
    }
    storage::clear_context_storage();
}

/// End the root span created in RINIT, recording the response status code (and any
/// fatal error) for HTTP requests. Does nothing if no root span was created.
fn close_root_span() {
    let Some(context_id) = OTEL_CONTEXT_ID.with(|cell| cell.borrow_mut().take()) else {
        tracing::debug!("RSHUTDOWN::not auto-closing root span...");
        return;
    };
    tracing::debug!("RSHUTDOWN::auto-closing root span...");
    let Some(ctx) = storage::get_context_instance(Some(context_id)) else {
        tracing::warn!("RSHUTDOWN::no context found for id {}", context_id);
        return;
    };
    let span = ctx.span();
    if !span.span_context().is_valid() {
        return;
    }
    if get_sapi_module_name() != "cli" {
        let response_code = get_response_status_code();
        span.set_attribute(KeyValue::new(SemConv::trace::HTTP_RESPONSE_STATUS_CODE, response_code as i64));
        if response_code >= 500 {
            let mut func = ZVal::from("error_get_last");
            let mut args: Vec<ZVal> = Vec::new();
            let error = ZVal::call(&mut func, &mut args).ok();
            if let Some(error) = error
                && error.get_type_info().is_array() {
                tracing::debug!("RSHUTDOWN::HTTP error detected: {:?}", error);
                let attributes = php_error_to_attributes(&error);
                span.add_event("exception", attributes);
            }
            // https://opentelemetry.io/docs/specs/semconv/http/http-spans/#status
            span.set_status(opentelemetry::trace::Status::error(""));
        }
    }
    span.end();
    tracing::debug!("RSHUTDOWN::removing context: {}", context_id);
    drop(ctx);
    storage::maybe_remove_context_instance(Some(context_id));
}

/// Ensure $_SERVER is initialized, which may not be true if auto_globals_jit is enabled.
fn jit_initialization() {
    unsafe {
        // `auto_globals_jit` is `zend_bool` (a `u8`) on PHP 7.x but a real `bool` on PHP 8+,
        // so neither a `u8` annotation nor a `bool` one compiles everywhere, and converting
        // between them is a useless conversion (which clippy denies) on whichever version
        // already has the target type. Comparing against `Default::default()` infers the
        // field's own type, so it is both correct and lint-clean on every version.
        let jit_initialization = pg!(auto_globals_jit);
        if jit_initialization != Default::default() {
            tracing::debug!("JIT auto_globals_jit enabled, initializing $_SERVER");
            let mut server = "_SERVER".to_string();
            sys::zend_is_auto_global_str(server.as_mut_ptr().cast(), server.len());
        }
    }
}

// @see https://github.com/apache/skywalking-php/blob/v0.8.0/src/request.rs#L152
#[allow(static_mut_refs)]
fn get_request_server<'a>() -> anyhow::Result<&'a ZArr> {
    unsafe {
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
                    if let IterKey::ZStr(zstr) = key
                        && let Ok(key_str) = zstr.to_str()
                        && key_str.starts_with(prefix)
                        && let Some(value_str) = z_val_to_string(value) {
                        return Some((key_str.to_string(), value_str));
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
        .and_then(z_val_to_string)
}

fn extract_request_headers(server: &ZArr) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    for (key, value) in server.iter() {
        if let IterKey::ZStr(zstr) = key
            && let Ok(key_str) = zstr.to_str()
            && key_str.starts_with("HTTP_")
            && let Some(value_str) = value.as_z_str().and_then(|z| z.to_str().ok()) {
            // Convert HTTP_ header names to standard format (e.g., HTTP_USER_AGENT -> User-Agent)
            let header_name = key_str
                .trim_start_matches("HTTP_")
                .replace('_', "-")
                .to_ascii_lowercase();

            headers.insert(header_name, value_str.to_string());
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
fn set_request_dotenv(env: HashMap<String, String>) {
    for (k, v) in env {
        unsafe { std::env::set_var(&k, &v) };
        tracing::debug!("Set environment variable from .env {}={}", k, v);
    }
}

fn backup_env() {
    tracing::debug!("Backing up OTEL_* environment variables");
    let env = env::vars()
        .filter(|(k, _)| k.starts_with("OTEL_"))
        .collect::<HashMap<_, _>>();
    ENV_BACKUP.with(|slot| *slot.borrow_mut() = Some(env));
}

fn restore_env() {
    tracing::debug!("Restoring environment variables from backup");
    if let Some(backup) = ENV_BACKUP.with(|slot| slot.borrow_mut().take()) {
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
#[cfg(test)]
mod tests {
    use super::*;

    /// `is_excluded_url` reads the environment, so the matching logic is tested through the
    /// compiled regex directly.
    fn matches(list: &str, uri: &str) -> bool {
        match compile_excluded_urls(list) {
            Some(re) => re.is_match(uri),
            None => false,
        }
    }

    #[test]
    fn exact_patterns_match_the_whole_uri() {
        assert!(matches("/health", "/health"));
        assert!(!matches("/health", "/health-check"));
        assert!(!matches("/health", "/foo/health"));
    }

    #[test]
    fn wildcards_match_prefix_suffix_and_middle() {
        assert!(matches("/health*", "/health-check"));
        assert!(matches("*.css", "/assets/site.css"));
        assert!(matches("/api/*/status", "/api/v2/status"));
        assert!(!matches("/api/*/status", "/api/v2/health"));
    }

    #[test]
    fn a_bare_wildcard_matches_everything() {
        assert!(matches("*", "/anything/at/all"));
    }

    #[test]
    fn any_pattern_in_the_list_can_match() {
        // the list from tests/http/ignore-url.phpt
        assert!(matches("/health*,/foo/bar", "/health-check"));
        assert!(matches("/health*,/foo/bar", "/foo/bar"));
        assert!(!matches("/health*,/foo/bar", "/foo/baz"));
    }

    #[test]
    fn whitespace_and_empty_entries_are_ignored() {
        assert!(matches(" /health , , /foo ", "/health"));
        assert!(matches(" /health , , /foo ", "/foo"));
        assert!(!matches(",,", "/health"));
        assert!(!matches("", "/health"));
    }

    #[test]
    fn regex_metacharacters_are_matched_literally() {
        // '.' must not match an arbitrary character...
        assert!(matches("/a.b", "/a.b"));
        assert!(!matches("/a.b", "/axb"));
        // ...and a pattern that is not valid regex must still work rather than blow up
        assert!(matches("/foo(1)", "/foo(1)"));
        assert!(matches("/foo[", "/foo["));
        assert!(matches("/q?x=1", "/q?x=1"));
        assert!(!matches("/q?x=1", "/qx=1"));
    }

    #[test]
    fn parses_resource_attributes() {
        let attrs = parse_resource_attributes("service.name=foo,service.version=1.0");
        assert_eq!(attrs.get("service.name"), Some(&"foo".to_string()));
        assert_eq!(attrs.get("service.version"), Some(&"1.0".to_string()));
        assert_eq!(attrs.len(), 2);
    }

    #[test]
    fn resource_attribute_values_may_contain_equals() {
        let attrs = parse_resource_attributes("k=a=b");
        assert_eq!(attrs.get("k"), Some(&"a=b".to_string()));
    }

    #[test]
    fn resource_attributes_are_trimmed_and_incomplete_pairs_dropped() {
        let attrs = parse_resource_attributes(" a = 1 ,novalue,=noKey,b=2,");
        assert_eq!(attrs.get("a"), Some(&"1".to_string()));
        assert_eq!(attrs.get("b"), Some(&"2".to_string()));
        assert_eq!(attrs.len(), 2);
    }

    #[test]
    fn empty_resource_attributes_yield_nothing() {
        assert!(parse_resource_attributes("").is_empty());
    }
}
