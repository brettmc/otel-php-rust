use crate::{
    context::{
        context::{make_context_class},
    },
    trace::{
        //scope::{make_scope_class},
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
    observer::{register_exec_functions},
};
use phper::{
    modules::Module,
    php_get_module,
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
pub mod observer;

static TRACER_PROVIDER: OnceLock<Arc<SdkTracerProvider>> = OnceLock::new();
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

#[php_get_module]
pub fn get_module() -> Module {
    let mut module = Module::new(
        env!("CARGO_CRATE_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
    );

    //module.add_class(make_scope_class());
    module.add_class(make_current_span_class());
    module.add_class(make_context_class());
    module.add_class(make_tracer_provider_class());
    module.add_class(make_tracer_class());
    module.add_class(make_span_class());
    module.add_class(make_span_builder_class());
    module.add_class(make_span_context_class());
    module.add_class(make_globals_class());
    module.add_class(make_status_code_class());

    module.on_module_init(|| {
        //TODO: configure internal logging, redirect to php error log?
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        RUNTIME.set(runtime).expect("Failed to store Tokio runtime");

        global::set_text_map_propagator(TraceContextPropagator::new());
        let provider = get_tracer_provider().clone();
        let _ = TRACER_PROVIDER.set(provider.clone());
        global::set_tracer_provider((*provider).clone());

        register_exec_functions();
    });
    module.on_module_shutdown(|| {
        if let Some(provider) = TRACER_PROVIDER.get() {
            let _ = provider.shutdown();
        }
        //not required? should be dropped on shutdown
        // if let Some(runtime) = RUNTIME.get() {
        //     let _ = runtime; //drop runtime
        // }
    });

    module
}

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get().expect("Tokio runtime not initialized")
}
