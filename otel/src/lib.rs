use crate::{
    context::{
        context::{build_context_class, new_context_class},
        context_interface::{make_context_interface},
        context_storage_interface::{make_context_storage_interface},
        scope::{build_scope_class, new_scope_class},
        scope_interface::make_scope_interface,
        storage::{build_storage_class, new_storage_class},
        propagation::{
            text_map_propagator_interface::{make_text_map_propagator_interface},
        }
    },
    trace::{
        local_root_span::make_local_root_span_class,
        non_recording_span::make_non_recording_span_class,
        span::{make_span_class},
        span_interface::make_span_interface,
        span_builder::{make_span_builder_class},
        status_code::{make_status_code_interface},
        tracer::{make_tracer_class},
        tracer_interface::{make_tracer_interface},
        tracer_provider,
        tracer_provider::{
            make_tracer_provider_class,
        },
        tracer_provider_interface::{make_tracer_provider_interface},
        span_context::{make_span_context_class},
        propagation::{
            trace_context_propagator::{make_trace_context_propagator_class},
        },
    },
    globals::{make_globals_class},
};
use phper::{
    ini::Policy,
    modules::Module,
    php_get_module,
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
    pub mod context_interface;
    pub mod context_storage_interface;
    pub mod scope;
    pub mod scope_interface;
    pub mod storage;
    pub mod propagation{
        pub mod text_map_propagator_interface;
    }
}
pub mod trace{
    pub mod local_root_span;
    pub mod non_recording_span;
    pub mod span;
    pub mod span_interface;
    pub mod span_builder;
    pub mod span_context;
    pub mod status_code;
    pub mod tracer;
    pub mod tracer_interface;
    pub mod tracer_provider;
    pub mod tracer_provider_interface;
    pub mod plugin_manager;
    pub mod plugin;
    pub mod plugins{
        pub mod psr18;
        pub mod test;
    }
    pub mod propagation{
        pub mod trace_context_propagator;
    }
}
pub mod globals;
pub mod request;
pub mod logging;
pub mod util;

// conditional compilation for observer feature (php8+)
#[cfg(feature = "php_observer")]
pub mod observer;
#[cfg(feature = "php_observer")]
use crate::trace::plugin_manager::PluginManager;
#[cfg(feature = "php_observer")]
use phper::sys;

#[cfg(feature = "php_execute")]
pub mod execute;

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
    module.add_ini("otel.cli.create_root_span", false, Policy::All);

    //interfaces
    let scope_interface = module.add_interface(make_scope_interface());
    let context_interface = module.add_interface(make_context_interface());
    let context_storage_interface = module.add_interface(make_context_storage_interface());
    let tracer_interface = module.add_interface(make_tracer_interface());
    let tracer_provider_interface = module.add_interface(make_tracer_provider_interface());
    let text_map_propagator_interface = module.add_interface(make_text_map_propagator_interface());
    let span_interface = module.add_interface(make_span_interface());

    //co-dependent classes
    let mut scope_class_entity = new_scope_class();
    let mut context_class_entity = new_context_class();
    let mut storage_class_entity = new_storage_class();
    build_scope_class(&mut scope_class_entity, &context_class_entity, &scope_interface);
    build_context_class(&mut context_class_entity, &scope_class_entity, &storage_class_entity, context_interface);
    build_storage_class(&mut storage_class_entity, &scope_class_entity, &context_class_entity, &context_storage_interface);

    let trace_context_propagator_class = module.add_class(make_trace_context_propagator_class(text_map_propagator_interface, &context_class_entity));
    let span_context_class = module.add_class(make_span_context_class());
    let scope_class = module.add_class(scope_class_entity);
    let context_class = module.add_class(context_class_entity);
    let _storage_class = module.add_class(storage_class_entity);

    let span_class = module.add_class(make_span_class(scope_class.clone(), span_context_class.clone(), context_class.clone(), &span_interface));
    let non_recording_span_class = module.add_class(make_non_recording_span_class(scope_class.clone(), span_context_class.clone(), context_class.clone(), &span_interface));
    let span_builder_class = module.add_class(make_span_builder_class(span_class.clone()));
    let _local_root_span_class = module.add_class(make_local_root_span_class(span_class.clone(), non_recording_span_class.clone()));

    let tracer_class = module.add_class(make_tracer_class(span_builder_class.clone(), tracer_interface));
    let tracer_provider_class = module.add_class(make_tracer_provider_class(tracer_class.clone(), tracer_provider_interface));
    let _globals_class = module.add_class(make_globals_class(tracer_provider_class.clone(), trace_context_propagator_class.clone()));
    let _status_code_interface = module.add_interface(make_status_code_interface());

    module.on_module_init(|| {
        logging::print_message("OpenTelemetry::MINIT".to_string());

        #[cfg(feature = "php_observer")]
        {
            observer::init(PluginManager::new());
            //todo: do this in observer.rs ?
            unsafe {
                sys::zend_observer_fcall_register(Some(observer::observer_instrument));
            }
            logging::print_message("registered fcall handlers".to_string());
        }
        #[cfg(feature = "php_execute")]
        {
            execute::register_exec_functions();
            //PluginManager::init();
        }
    });
    module.on_module_shutdown(|| {
        logging::print_message("OpenTelemetry::MSHUTDOWN".to_string());
        tracer_provider::shutdown();
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
