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
    ini::Policy,
    modules::Module,
    php_get_module,
    sys,
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
pub mod request;
pub mod observer;
pub mod logging;

static TRACER_PROVIDER: OnceLock<Arc<SdkTracerProvider>> = OnceLock::new();
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

#[php_get_module]
pub fn get_module() -> Module {
    let mut module = Module::new(
        env!("CARGO_CRATE_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
    );
    module.add_info("opentelemetry-rust", "0.28.0");
    module.add_ini("otel.log_level", "warn".to_string(), Policy::All);

    let span_context_class = module.add_class(make_span_context_class());
    let scope_class = module.add_class(make_scope_class());
    let current_span_class = module.add_class(make_current_span_class(span_context_class.clone()));
    let _context_class = module.add_class(make_context_class());
    let span_class = module.add_class(make_span_class(scope_class.clone(), span_context_class.clone(), current_span_class.clone()));
    let span_builder_class = module.add_class(make_span_builder_class(span_class.clone()));

    let tracer_class = module.add_class(make_tracer_class(span_builder_class.clone()));
    let tracer_provider_class = module.add_class(make_tracer_provider_class(tracer_class.clone()));
    let _globals_class = module.add_class(make_globals_class(tracer_provider_class.clone()));
    let _status_code_class = module.add_class(make_status_code_class());

    module.on_module_init(|| {
        logging::init();

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
        request::init();
    });
    module.on_request_shutdown(|| {
        request::shutdown();
    });

    module
}

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get().expect("Tokio runtime not initialized")
}
