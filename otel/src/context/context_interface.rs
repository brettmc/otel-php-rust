use phper::{
    classes::{InterfaceEntity},
    functions::{ReturnType},
    types::{ReturnTypeHint},
};
use crate::context::scope_interface::SCOPE_INTERFACE_NAME;

pub const CONTEXT_INTERFACE_NAME: &str = r"OpenTelemetry\Context\ContextInterface";

pub fn make_context_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(CONTEXT_INTERFACE_NAME);
    interface
        .add_static_method("getCurrent")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME))));
    interface
        .add_method("activate")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(SCOPE_INTERFACE_NAME))));

    interface
}
