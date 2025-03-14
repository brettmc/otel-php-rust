use crate::{
    context::{
        context::{make_context_class},
    },
    trace::{
        plugin_manager::PluginManager,
        scope::{make_scope_class},
        span::{make_span_class},
        span_builder::{make_span_builder_class},
        status_code::{make_status_code_class},
        tracer::{make_tracer_class},
        tracer_provider,
        tracer_provider::{
            make_tracer_provider_class,
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
use std::{
    sync::{
        OnceLock,
    },
};
use opentelemetry::{
    global,
};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
};
use tokio::runtime::Runtime;

pub mod context{
    pub mod context;
}
pub mod trace{
    pub mod scope;
    pub mod span;
    pub mod span_builder;
    pub mod span_context;
    pub mod status_code;
    pub mod tracer;
    pub mod tracer_provider;
    pub mod plugin_manager;
    pub mod plugin;
    pub mod plugins{
        pub mod test;
    }
}
pub mod globals;
pub mod request;
pub mod observer;
pub mod logging;

static RUNTIME: OnceLock<Runtime> = OnceLock::new(); //TODO one runtime per PID?

#[php_get_module]
pub fn get_module() -> Module {
    let mut module = Module::new(
        env!("CARGO_CRATE_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
    );
    module.add_info("opentelemetry-rust", "0.28.0");
    module.add_ini("otel.log.level", "error".to_string(), Policy::All);
    module.add_ini("otel.log.file", "/var/log/ext-otel.log".to_string(), Policy::All);

    let span_context_class = module.add_class(make_span_context_class());
    let scope_class = module.add_class(make_scope_class());
    let _context_class = module.add_class(make_context_class());
    let span_class = module.add_class(make_span_class(scope_class.clone(), span_context_class.clone()));
    let span_builder_class = module.add_class(make_span_builder_class(span_class.clone()));

    let tracer_class = module.add_class(make_tracer_class(span_builder_class.clone()));
    let tracer_provider_class = module.add_class(make_tracer_provider_class(tracer_class.clone()));
    let _globals_class = module.add_class(make_globals_class(tracer_provider_class.clone()));
    let _status_code_class = module.add_class(make_status_code_class());

    module.on_module_init(|| {
        logging::print_message("OpenTelemetry::MINIT".to_string());
        observer::init(PluginManager::new());
        unsafe {
            sys::zend_observer_fcall_register(Some(observer::observer_instrument));
        }
        logging::print_message("registered fcall handlers".to_string());

        //TODO use this if multiple grpc exporters (eg logging, metrics)
        // let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        // RUNTIME.set(runtime).expect("Failed to store Tokio runtime");

        //global::set_text_map_propagator(TraceContextPropagator::new());
        //let provider = get_tracer_provider().clone();
        //let _ = TRACER_PROVIDER.set(provider.clone());
        //global::set_tracer_provider((*provider).clone());
    });
    module.on_module_shutdown(|| {
        logging::print_message("OpenTelemetry::MSHUTDOWN".to_string());
        // tracing::trace!("MSHUTDOWN");
        tracer_provider::shutdown(); //TODO this already runs after MSHUTDOWN (at least in cli) ??
        /*if let Some(provider) = TRACER_PROVIDER.get() {
            let shutdown_result = provider.shutdown();
            match shutdown_result {
                Ok(_) => tracing::debug!("MSHUTDOWN::OpenTelemetry tracer provider shutdown success"),
                Err(err) => tracing::warn!("MSHUTDOWN::Failed to shutdown OpenTelemetry tracer provider: {:?}", err),
            }
        }*/
    });
    module.on_request_init(|| {
        logging::print_message("OpenTelemetry::RINIT".to_string());
        logging::init_once();
        // tracing::trace!("RINIT");
        tracer_provider::init_once();
        global::set_text_map_propagator(TraceContextPropagator::new()); //TODO could this be lazy-loaded?

        request::init();
    });
    module.on_request_shutdown(|| {
        logging::print_message("OpenTelemetry::RSHUTDOWN".to_string());
        // tracing::trace!("RSHUTDOWN");
        request::shutdown();
    });

    module
}

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get().expect("Tokio runtime not initialized")
}
