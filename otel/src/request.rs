use anyhow::Context as _;
use phper::{
    eg,
    ini::ini_get,
    sg,
    sys,
    sys::sapi_module,
    arrays::{IterKey, ZArr},
    values::ZVal,
};
use std::{
    cell::RefCell,
    ffi::CStr,
    collections::HashMap,
    sync::Arc,
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
    context::storage,
    trace::{local_root_span, tracer_provider},
};

thread_local! {
    static OTEL_REQUEST_GUARD: RefCell<Option<opentelemetry::ContextGuard>> = RefCell::new(None);
    static OTEL_CONTEXT_ID: RefCell<Option<u64>> = RefCell::new(None);
}

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
        let trace_cli = ini_get::<bool>("otel.cli.create_root_span");
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
    let scope = InstrumentationScope::builder("php_request").build();
    let tracer = tracer_provider.tracer_with_scope(scope);
    let span_builder = tracer.span_builder(span_name.unwrap_or("unknown".to_string()));
    let mut attributes = span_builder.attributes.clone().unwrap_or_default();
    attributes.push(KeyValue::new("php.sapi.name", get_sapi_module_name()));
    attributes.push(KeyValue::new(SemConv::trace::URL_FULL, request_details.uri.unwrap_or_default()));
    attributes.push(KeyValue::new(SemConv::trace::HTTP_REQUEST_METHOD, request_details.method.unwrap_or_default()));
    //attributes.push(KeyValue::new(SemConv::trace::HTTP_REQUEST_BODY_SIZE, request_details.body_length.unwrap_or_default()));
    // TODO set other span attributes from request

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
        local_root_span::store_local_root_span(context_id);
    }
    //TODO use span::storeInContext logic
    let guard = ctx.attach();

    OTEL_REQUEST_GUARD.with(|slot| {
        *slot.borrow_mut() = Some(guard);
    });
}

pub fn shutdown() {
    let context_id = OTEL_CONTEXT_ID.with(|cell| cell.borrow_mut().take());
    let is_tracing = context_id.is_some();
    let sapi = get_sapi_module_name();
    if is_tracing {
        let context_id = context_id.unwrap();
        let is_http_request = sapi != "cli";
        tracing::debug!("RSHUTDOWN::auto-closing root span...");
        let ctx = storage::get_context_instance(context_id).unwrap();
        let span = ctx.span();
        if span.span_context().is_valid() {
            if is_http_request {
                let response_code = get_response_status_code();
                span.set_attribute(KeyValue::new(SemConv::trace::HTTP_RESPONSE_STATUS_CODE, response_code as i64));
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
}

fn get_sapi_module_name() -> String {
    unsafe { CStr::from_ptr(sapi_module.name).to_string_lossy().into_owned() }
}

fn get_request_details() -> RequestDetails {
    unsafe {
        let request_info = sg!(request_info);
        RequestDetails {
            method: Some(request_info.request_method)
                .filter(|ptr| !ptr.is_null())
                .map(|ptr| std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()),
            uri: Some(request_info.request_uri)
                .filter(|ptr| !ptr.is_null())
                .map(|ptr| std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()),
            body_length: request_info.content_length as u64,
            content_type: Some(request_info.content_type)
                .filter(|ptr| !ptr.is_null())
                .map(|ptr| std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()),
        }
    }
}

#[allow(dead_code)]
struct RequestDetails {
    method: Option<String>,
    uri: Option<String>,
    body_length: u64,
    content_type: Option<String>,
}

// @see https://github.com/apache/skywalking-php/blob/v0.8.0/src/request.rs#L152
#[allow(static_mut_refs)]
pub fn get_request_server<'a>() -> anyhow::Result<&'a ZArr> {
    unsafe {
        let symbol_table = ZArr::from_mut_ptr(&mut eg!(symbol_table));
        let carrier = symbol_table
            .get("_SERVER")
            .and_then(|carrier| carrier.as_z_arr())
            .context("$_SERVER is null")?;
        Ok(carrier)
    }
}

pub fn extract_request_headers(server: &ZArr) -> HashMap<String, String> {
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

pub fn z_val_to_string(zv: &ZVal) -> Option<String> {
    zv.as_z_str()
        .and_then(|zs| zs.to_str().ok())
        .map(|s| s.to_string())
}

fn get_response_status_code() -> i32 {
    unsafe { sg!(sapi_headers).http_response_code }
}

pub fn get_propagated_context() -> Context {
    let server = match get_request_server() {
        Ok(server) => server,
        Err(_) => return Context::current(),
    };

    // Extract headers from `$_SERVER`
    let headers = extract_request_headers(server);

    global::get_text_map_propagator(|prop| prop.extract(&headers))
}