use phper::{
    classes::{InterfaceEntity},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};

pub fn make_context_storage_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(r"OpenTelemetry\Context\ContextStorageInterface");
    interface
        .add_method("scope")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ScopeInterface"))).allow_null()); //ContextStorageScopeInterface

    interface
        .add_static_method("current")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ContextInterface"))));

    interface
        .add_method("attach")
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ContextInterface"))))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ScopeInterface")))); //ContextStorageScopeInterface

    interface
}
