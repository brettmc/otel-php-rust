use phper::{
    classes::{InterfaceEntity},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use crate::context::context_interface::CONTEXT_INTERFACE_NAME;
use crate::context::scope_interface::SCOPE_INTERFACE_NAME;

pub const CONTEXT_STORAGE_INTERFACE_NAME: &str = r"OpenTelemetry\Context\ContextStorageInterface";

pub fn make_context_storage_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(CONTEXT_STORAGE_INTERFACE_NAME);
    interface
        .add_method("scope")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(SCOPE_INTERFACE_NAME))).allow_null()); //ContextStorageScopeInterface

    interface
        .add_static_method("current")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME))));

    interface
        .add_method("attach")
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME))))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(SCOPE_INTERFACE_NAME)))); //ContextStorageScopeInterface

    interface
}
