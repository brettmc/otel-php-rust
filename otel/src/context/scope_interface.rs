use phper::{
    classes::{InterfaceEntity},
    functions::{ReturnType},
    types::{ReturnTypeHint},
};

pub fn make_scope_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(r"OpenTelemetry\Context\ScopeInterface");
    interface
        .add_method("detach")
        .return_type(ReturnType::new(ReturnTypeHint::Int));

    interface
        .add_method("context")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ContextInterface"))));

    interface
}
