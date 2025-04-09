use phper::{
    classes::{InterfaceEntity},
    functions::{ReturnType},
    types::{ReturnTypeHint},
};

pub fn make_context_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(r"OpenTelemetry\Context\ContextInterface");
    interface
        .add_static_method("getCurrent")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ContextInterface"))));
    interface
        .add_method("activate")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ScopeInterface"))));

    interface
}
