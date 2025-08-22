use phper::{
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
pub mod class_registry;
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

include!(concat!(env!("OUT_DIR"), "/package_versions.rs"));

#[php_get_module]
pub fn get_module() -> Module {
    let mut module = Module::new(
        env!("CARGO_CRATE_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
    );

    module::add_module_info(&mut module);
    module::add_module_ini(&mut module);

    class_registry::register_classes_and_interfaces(&mut module);

    module.on_module_init(module::on_module_init);
    module.on_module_shutdown(module::on_module_shutdown);
    module.on_request_init(request::on_request_init);
    module.on_request_shutdown(request::on_request_shutdown);

    module
}
