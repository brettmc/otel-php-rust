use crate::{
    context::{
        context::{make_context_class},
    },
    trace::{
        local_root_span::{make_local_root_span_class},
        plugin_manager::PluginManager,
        scope::{make_scope_class},
        span::{make_span_class},
        span_builder::{make_span_builder_class},
        status_code::{make_status_code_interface},
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
use opentelemetry::{
    global,
};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
};
use tokio::runtime::Runtime;
use once_cell::sync::OnceCell;

pub mod context{
    pub mod context;
}
pub mod trace{
    pub mod local_root_span;
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
        pub mod psr18;
        pub mod test;
    }
}
pub mod globals;
pub mod request;
pub mod observer;
pub mod logging;
pub mod util;

include!(concat!(env!("OUT_DIR"), "/package_versions.rs"));

static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();

#[php_get_module]
pub fn get_module() -> Module {
    let mut module = Module::new(
        env!("CARGO_CRATE_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
    );
    module.add_info("opentelemetry-rust", OPENTELEMETRY_VERSION);
    module.add_info("phper", PHPER_VERSION);
    module.add_info("tokio", TOKIO_VERSION);
    module.add_ini("otel.log.level", "error".to_string(), Policy::All);
    module.add_ini("otel.log.file", "/dev/stderr".to_string(), Policy::All);

    let span_context_class = module.add_class(make_span_context_class());
    let scope_class = module.add_class(make_scope_class());
    let _context_class = module.add_class(make_context_class());
    let span_class = module.add_class(make_span_class(scope_class.clone(), span_context_class.clone()));
    let span_builder_class = module.add_class(make_span_builder_class(span_class.clone()));
    let _local_root_span_class = module.add_class(make_local_root_span_class(span_class.clone()));

    let tracer_class = module.add_class(make_tracer_class(span_builder_class.clone()));
    let tracer_provider_class = module.add_class(make_tracer_provider_class(tracer_class.clone()));
    let _globals_class = module.add_class(make_globals_class(tracer_provider_class.clone()));
    let _status_code_interface = module.add_interface(make_status_code_interface());

    module.on_module_init(|| {
        logging::print_message("OpenTelemetry::MINIT".to_string());

        observer::init(PluginManager::new());
        unsafe {
            sys::zend_observer_fcall_register(Some(observer::observer_instrument));
        }
        logging::print_message("registered fcall handlers".to_string());
    });
    module.on_module_shutdown(|| {
        logging::print_message("OpenTelemetry::MSHUTDOWN".to_string());
        tracer_provider::force_flush();
    });
    module.on_request_init(|| {
        logging::print_message("OpenTelemetry::RINIT".to_string());
        logging::init_once();

        if TOKIO_RUNTIME.get().is_none() {
            logging::print_message("OpenTelemetry::RINIT::Creating tokio runtime".to_string());
            //TODO don't create runtime unless using grpc
            let runtime = Runtime::new().expect("Failed to create Tokio runtime");
            TOKIO_RUNTIME.set(runtime).expect("Tokio runtime already set");
            logging::print_message("OpenTelemetry::RINIT::tokio runtime initialized".to_string());
        }

        tracer_provider::init_once();
        global::set_text_map_propagator(TraceContextPropagator::new());

        request::init();
    });
    module.on_request_shutdown(|| {
        logging::print_message("OpenTelemetry::RSHUTDOWN".to_string());
        request::shutdown();
    });

    module
}

pub fn get_runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get().expect("Tokio runtime not initialized")
}
