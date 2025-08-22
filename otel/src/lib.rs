use phper::{
    ini::Policy,
    modules::Module,
    php_get_module,
};
use std::env;

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
pub mod error;
pub mod globals;
pub mod request;
pub mod logging;
pub mod util;
pub mod module;

pub mod auto{
    #[cfg(otel_observer_supported)]
    pub mod observer;
    #[cfg(otel_observer_not_supported)]
    pub mod execute;
    pub mod execute_data;
    pub mod plugin_manager;
    pub mod utils;
    pub mod plugin;
    pub mod plugins{
        pub mod laminas;
        pub mod psr18;
        #[cfg(feature="test")]
        pub mod test;
        pub mod zf1;
    }
}

pub mod class_registry; // add this line

include!(concat!(env!("OUT_DIR"), "/package_versions.rs"));

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
    module.add_ini(config::ini::OTEL_ENV_DOTENV_ENABLED, false, Policy::All);
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

    class_registry::register_classes_and_interfaces(&mut module); // Move all class/interface generation to class_registry.rs

    module.on_module_init(module::on_module_init);
    module.on_module_shutdown(module::on_module_shutdown);
    module.on_request_init(request::on_request_init);
    module.on_request_shutdown(request::on_request_shutdown);

    module
}
