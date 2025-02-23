use phper::{
    sg,
    sys::sapi_module,
};
use std::cell::RefCell;
use std::ffi::CStr;
use opentelemetry::{
    Context,
    global,
    trace::{SpanKind, Tracer, TraceContextExt},
};

thread_local! {
    static OTEL_REQUEST_GUARD: RefCell<Option<opentelemetry::ContextGuard>> = RefCell::new(None);
}

pub fn init() {
    let sapi = get_sapi_module_name();
    if sapi == "cli" {
        return;
    }
    let span_name = match get_request_method() {
        Some(method) => format!("HTTP {}", method),
        None => "HTTP".to_string(),
    };
    let tracer = global::tracer("php_request");
    let mut span_builder = tracer.span_builder(span_name);
    span_builder.span_kind = Some(SpanKind::Server);
    // TODO set other span attributes from request
    let span = tracer.build_with_context(span_builder, &Context::current());
    let ctx = Context::current_with_span(span);
    let guard = ctx.attach();

    OTEL_REQUEST_GUARD.with(|slot| {
        *slot.borrow_mut() = Some(guard);
    });
}

pub fn shutdown() {
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