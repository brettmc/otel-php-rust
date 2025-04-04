use crate::{
    context::{
        context::{build_context_class, new_context_class},
        scope::{build_scope_class, new_scope_class},
    },
    trace::{
        plugin_manager::PluginManager,
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
    pub mod scope;
    pub mod storage;
}
pub mod trace{
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

    //co-dependent classes
    let mut scope_class_entity = new_scope_class();
    let mut context_class_entity = new_context_class();

    let span_context_class = module.add_class(make_span_context_class());
    build_scope_class(&mut scope_class_entity, &context_class_entity);
    build_context_class(&mut context_class_entity, &scope_class_entity);
    let scope_class = module.add_class(scope_class_entity);
    let context_class = module.add_class(context_class_entity);

    let span_class = module.add_class(make_span_class(scope_class, span_context_class.clone(), context_class.clone()));
    let span_builder_class = module.add_class(make_span_builder_class(span_class.clone()));

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
