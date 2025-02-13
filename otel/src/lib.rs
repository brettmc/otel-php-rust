use crate::{
    trace::{
        //scope::{make_scope_class},
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

pub mod trace{
    pub mod scope;
    pub mod span;
    pub mod span_builder;
    pub mod span_context;
    pub mod status_code;
    pub mod tracer;
    pub mod tracer_provider;
}
pub mod globals;

static TRACER_PROVIDER: OnceLock<Arc<SdkTracerProvider>> = OnceLock::new();

#[php_get_module]
pub fn get_module() -> Module {
    let mut module = Module::new(
        env!("CARGO_CRATE_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
    );

    //module.add_class(make_scope_class());
    module.add_class(make_tracer_provider_class());
    module.add_class(make_tracer_class());
    module.add_class(make_span_class());
    module.add_class(make_span_builder_class());
    module.add_class(make_span_context_class());
    module.add_class(make_globals_class());
    module.add_class(make_status_code_class());

    module.on_module_init(|| {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let provider = get_tracer_provider().clone();
        let _ = TRACER_PROVIDER.set(provider.clone());
        global::set_tracer_provider((*provider).clone());
    });
    module.on_module_shutdown(|| {
        if let Some(provider) = TRACER_PROVIDER.get() {
            let _ = provider.shutdown();
        }
    });

    module
}
