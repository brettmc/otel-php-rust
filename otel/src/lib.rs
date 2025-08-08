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
        memory_exporter::make_memory_exporter_class,
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
    util::get_sapi_module_name,
    globals::{make_globals_class},
};
use phper::{
    ini::{ini_get, Policy},
    modules::Module,
    php_get_module,
};
use opentelemetry::{
    global,
};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
};
use std::env;
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
    pub mod memory_exporter;
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
    pub mod propagation{
        pub mod trace_context_propagator;
    }
}
pub mod config{
    pub mod ini;
    pub mod trace_attributes;
}
pub mod globals;
pub mod request;
pub mod logging;
pub mod util;

pub mod auto{
    #[cfg(otel_observer_supported)]
    pub mod observer;
    #[cfg(otel_observer_not_supported)]
    pub mod execute;
    pub mod execute_data;
    pub mod plugin_manager;
    pub mod plugin;
    pub mod plugins{
        pub mod laminas;
        pub mod psr18;
        #[cfg(feature="test")]
        pub mod test;
        pub mod zf1;
    }
}
use crate::auto::plugin_manager::PluginManager;

include!(concat!(env!("OUT_DIR"), "/package_versions.rs"));

static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();
static DISABLED: OnceCell<bool> = OnceCell::new();

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
    #[cfg(otel_observer_supported)]
    module.add_info("auto-instrumentation", "observer_api");
    #[cfg(otel_observer_not_supported)]
    module.add_info("auto-instrumentation", "zend_execute_ex");
    module.add_ini(config::ini::OTEL_LOG_LEVEL, "error".to_string(), Policy::All);
    module.add_ini(config::ini::OTEL_LOG_FILE, "/dev/stderr".to_string(), Policy::All);
    module.add_ini(config::ini::OTEL_CLI_CREATE_ROOT_SPAN, false, Policy::All);
    module.add_ini(config::ini::OTEL_CLI_ENABLED, false, Policy::All);
    module.add_ini(config::ini::OTEL_DOTENV_PER_REQUEST, false, Policy::All);
    module.add_ini(config::ini::OTEL_ENV_SET_FROM_SERVER, false, Policy::All);
    module.add_ini(config::ini::OTEL_AUTO_ENABLED, true, Policy::All);
    module.add_ini(config::ini::OTEL_AUTO_DISABLED_PLUGINS, "".to_string(), Policy::All);
    //which auto-instrumentation mechanism is enabled
    #[cfg(otel_observer_supported)]
    {
        module.add_info("auto-instrumentation", "observer".to_string());
        module.add_constant("OTEL_AUTO_INSTRUMENTATION", "observer".to_string());
    }
    #[cfg(otel_observer_not_supported)]
    {
        module.add_info("auto-instrumentation", "zend_execute_ex".to_string());
        module.add_constant("OTEL_AUTO_INSTRUMENTATION", "zend_execute_ex".to_string());
    }

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
    let _in_memory_exporter_class = module.add_class(make_memory_exporter_class());

    let span_class = module.add_class(make_span_class(scope_class.clone(), span_context_class.clone(), context_class.clone(), &span_interface));
    let non_recording_span_class = module.add_class(make_non_recording_span_class(scope_class.clone(), span_context_class.clone(), context_class.clone(), &span_interface));
    let span_builder_class = module.add_class(make_span_builder_class(span_class.clone()));
    let _local_root_span_class = module.add_class(make_local_root_span_class(span_class.clone(), non_recording_span_class.clone()));

    let tracer_class = module.add_class(make_tracer_class(span_builder_class.clone(), tracer_interface));
    let tracer_provider_class = module.add_class(make_tracer_provider_class(tracer_class.clone(), tracer_provider_interface));
    let _globals_class = module.add_class(make_globals_class(tracer_provider_class.clone(), trace_context_propagator_class.clone()));
    let _status_code_interface = module.add_interface(make_status_code_interface());

    module.on_module_init(|| {
        logging::init_once(); //from here on we can use tracing macros
        let cli_enabled = ini_get::<bool>(config::ini::OTEL_CLI_ENABLED);
        let sapi = get_sapi_module_name();
        let disabled = sapi == "cli" && !cli_enabled;
        DISABLED.set(disabled).ok();
        if disabled {
            tracing::debug!("OpenTelemetry::MINIT disabled");
            return;
        }
        tracing::debug!("OpenTelemetry::MINIT");

        let auto_enabled = ini_get::<bool>(config::ini::OTEL_AUTO_ENABLED);
        if auto_enabled {
            let plugin_manager = PluginManager::new();

            #[cfg(otel_observer_supported)]
            {
                crate::auto::observer::init(plugin_manager);
            }
            #[cfg(otel_observer_not_supported)]
            {
                crate::auto::execute::init(plugin_manager);
            }
        } else {
            tracing::debug!("OpenTelemetry::MINIT auto-instrumentation disabled");
        }
    });
    module.on_module_shutdown(|| {
        let is_disabled = *DISABLED.get().unwrap_or(&false);
        if is_disabled {
            return;
        }
        tracing::debug!("OpenTelemetry::MSHUTDOWN");
        tracer_provider::shutdown();
    });
    module.on_request_init(|| {
        if *DISABLED.get().unwrap_or(&false) {
            return;
        }
        logging::init_once(); //we maybe need to initialize logging for each worker (apache, fpm)
        tracing::debug!("OpenTelemetry::RINIT");
        request::init_environment();

        if request::is_disabled() {
            tracing::debug!("OpenTelemetry::RINIT: OTEL_SDK_DISABLED is set to true, skipping initialization");
            return;
        }

        let protocol = env::var("OTEL_EXPORTER_OTLP_PROTOCOL").unwrap_or("grpc".to_string());
        if protocol == "grpc" {
            if TOKIO_RUNTIME.get().is_none() {
                tracing::debug!("OpenTelemetry::RINIT::Creating tokio runtime");
                let runtime = Runtime::new().expect("Failed to create Tokio runtime");
                TOKIO_RUNTIME.set(runtime).expect("Tokio runtime already set");
                tracing::debug!("OpenTelemetry::RINIT::tokio runtime initialized");
            }
        } else {
            tracing::debug!("OpenTelemetry::RINIT not creating tokio runtime for non-gRPC exporter");
        }

        tracer_provider::init_once();
        global::set_text_map_propagator(TraceContextPropagator::new());

        request::init();
    });
    module.on_request_shutdown(|| {
        let is_disabled = *DISABLED.get().unwrap_or(&false);
        if is_disabled {
            return;
        }
        tracing::debug!("OpenTelemetry::RSHUTDOWN");
        request::shutdown();
    });

    module
}

pub fn get_runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get().expect("Tokio runtime not initialized")
}
