use crate::{
    context::{
        context::{make_context_class},
    },
    trace::{
        scope::{make_scope_class},
        current_span::{make_current_span_class},
        span::{make_span_class},
        span_builder::{make_span_builder_class},
        status_code::{make_status_code_class},
        tracer::{make_tracer_class},
        tracer_provider::{
            make_tracer_provider_class,
            get_tracer_provider,
        },
        span_context::{make_span_context_class},
    },
    globals::{make_globals_class},
};
use phper::{
    sg,
    functions::call,
    modules::Module,
    php_get_module,
    sys::sapi_module,
    sys,
    arrays::ZArr,
    values::{ZVal},
};
use std::sync::{
    Arc,
    OnceLock,
};
use opentelemetry::{
    global,
};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    trace::SdkTracerProvider,
};
use tokio::runtime::Runtime;
use std::ffi::CStr;
use opentelemetry::trace::SpanKind;

pub mod context{
    pub mod context;
}
pub mod trace{
    pub mod current_span;
    pub mod scope;
    pub mod span;
    pub mod span_builder;
    pub mod span_context;
    pub mod status_code;
    pub mod tracer;
    pub mod tracer_provider;
}
pub mod globals;
pub mod observer;
use opentelemetry::trace::Tracer;
use opentelemetry::Context;
use opentelemetry::trace::TraceContextExt;
use std::cell::RefCell;
use std::ptr;

static TRACER_PROVIDER: OnceLock<Arc<SdkTracerProvider>> = OnceLock::new();
static RUNTIME: OnceLock<Runtime> = OnceLock::new();
thread_local! {
    static OTEL_REQUEST_GUARD: RefCell<Option<opentelemetry::ContextGuard>> = RefCell::new(None);
}

#[php_get_module]
pub fn get_module() -> Module {
    let mut module = Module::new(
        env!("CARGO_CRATE_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
    );

    let span_context_class = module.add_class(make_span_context_class());
    let scope_class = module.add_class(make_scope_class());
    let current_span_class = module.add_class(make_current_span_class(span_context_class.clone()));
    let _context_class = module.add_class(make_context_class());
    let span_class = module.add_class(make_span_class(span_context_class, current_span_class.clone()));
    let span_builder_class = module.add_class(make_span_builder_class(span_class));

    let tracer_class = module.add_class(make_tracer_class(span_builder_class, scope_class));
    let tracer_provider_class = module.add_class(make_tracer_provider_class(tracer_class));
    let _globals_class = module.add_class(make_globals_class(tracer_provider_class));
    let _status_code_class = module.add_class(make_status_code_class());

    module.on_module_init(|| {
        //TODO: configure internal logging, redirect to php error log?
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        RUNTIME.set(runtime).expect("Failed to store Tokio runtime");

        global::set_text_map_propagator(TraceContextPropagator::new());
        let provider = get_tracer_provider().clone();
        let _ = TRACER_PROVIDER.set(provider.clone());
        global::set_tracer_provider((*provider).clone());

        unsafe {
            sys::zend_observer_fcall_register(Some(observer::observer_instrument));
        }
    });
    module.on_module_shutdown(|| {
        if let Some(provider) = TRACER_PROVIDER.get() {
            let _ = provider.shutdown();
        }
    });
    module.on_request_init(|| {
        // TODO move all of this into a new file, request.rs
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
    });
    module.on_request_shutdown(|| {
        OTEL_REQUEST_GUARD.with(|slot| {
            *slot.borrow_mut() = None;
        });
    });

    module
}

fn get_sapi_module_name() -> String {
    unsafe { CStr::from_ptr(sapi_module.name).to_string_lossy().into_owned() }
}

fn get_request_method() -> Option<String> {
    unsafe {
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
}

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get().expect("Tokio runtime not initialized")
}
