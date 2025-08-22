use phper::{
    modules::Module,
    php_get_module,
};
use std::env;

pub mod context;
pub mod trace;
pub mod class_registry;
pub mod config;
pub mod error;
pub mod globals;
pub mod request;
pub mod logging;
pub mod util;
pub mod module;
pub mod auto;

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
