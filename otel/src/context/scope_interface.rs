use phper::{
    classes::{InterfaceEntity},
    functions::{ReturnType},
    types::{ReturnTypeHint},
};
use crate::context::context_interface::CONTEXT_INTERFACE_NAME;

pub const SCOPE_INTERFACE_NAME: &str = r"OpenTelemetry\Context\ScopeInterface";

pub fn make_scope_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(SCOPE_INTERFACE_NAME);
    interface
        .add_method("detach")
        .return_type(ReturnType::new(ReturnTypeHint::Int));

    interface
        .add_method("context")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME))));

    interface
}
