use anyhow::Context as _;
use phper::{
    eg,
    sg,
    sys,
    sys::sapi_module,
    arrays::{IterKey, ZArr},
    values::ZVal,
};
use std::cell::RefCell;
use std::ffi::CStr;
use opentelemetry::{
    Context,
    InstrumentationScope,
    KeyValue,
    global,
    trace::{SpanKind, Tracer, TraceContextExt, TracerProvider},
};
use opentelemetry_semantic_conventions as SemConv;
use std::collections::HashMap;
use crate::trace::tracer_provider;

thread_local! {
    static OTEL_REQUEST_GUARD: RefCell<Option<opentelemetry::ContextGuard>> = RefCell::new(None);
}

pub fn init() {
    tracing::debug!("RINIT::initializing");
    unsafe {
        //ensure $_SERVER is populated
        let mut server = "_SERVER".to_string();
        sys::zend_is_auto_global_str(server.as_mut_ptr().cast(), server.len());
    }
    let sapi = get_sapi_module_name();
    tracing::debug!("RINIT::sapi module name is: {}", sapi.clone());
    //TODO apache2handler (mod_php) doesn't run RINIT/RSHUTDOWN
    if sapi == "cli" {
        tracing::debug!("RINIT::not auto-creating root span...");
        return;
    }
    let span_name = match get_request_method() {
        Some(method) => format!("HTTP {}", method),
        None => "HTTP".to_string(),
    };
    tracing::debug!("RINIT::otel request is being traced, name={}", span_name.clone());
    let tracer_provider = tracer_provider::get_tracer_provider();
    let scope = InstrumentationScope::builder("php_request").build();
    let tracer = tracer_provider.tracer_with_scope(scope);
    let mut span_builder = tracer.span_builder(span_name);
    span_builder.span_kind = Some(SpanKind::Server);
    let parent_context = get_propagated_context();
    // TODO set other span attributes from request
    let span = tracer.build_with_context(span_builder, &parent_context);
    let ctx = Context::current_with_span(span);
    let guard = ctx.attach();

    OTEL_REQUEST_GUARD.with(|slot| {
        *slot.borrow_mut() = Some(guard);
    });
}

pub fn shutdown() {
    tracing::debug!("RSHUTDOWN::maybe closing root span...");
    let response_code = get_response_status_code();
    let ctx = Context::current();
    let span = ctx.span();
    if span.span_context().is_valid() {
        span.set_attribute(KeyValue::new(SemConv::trace::HTTP_RESPONSE_STATUS_CODE, response_code as i64));
    }

    OTEL_REQUEST_GUARD.with(|slot| {
        *slot.borrow_mut() = None;
    });
}

fn get_sapi_module_name() -> String {
    unsafe { CStr::from_ptr(sapi_module.name).to_string_lossy().into_owned() }
}

fn get_request_method() -> Option<String> {
    unsafe {
        let request_info = sg!(request_info);
        let method_ptr = request_info.request_method;

        if !method_ptr.is_null() {
            Some(std::ffi::CStr::from_ptr(method_ptr).to_string_lossy().into_owned())
        } else {
            None
        }
    }
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