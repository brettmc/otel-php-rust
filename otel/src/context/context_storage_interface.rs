use crate::context::context_interface::CONTEXT_INTERFACE_NAME;
use crate::context::context_storage_scope_interface::CONTEXT_STORAGE_SCOPE_INTERFACE_NAME;
use phper::{
    classes::InterfaceEntity,
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};

pub const CONTEXT_STORAGE_INTERFACE_NAME: &str = r"OpenTelemetry\Context\ContextStorageInterface";

pub fn make_context_storage_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(CONTEXT_STORAGE_INTERFACE_NAME);

    // All three return `ContextStorageScopeInterface`, not the broader `ScopeInterface`: a
    // storage hands out scopes that can be asked for their context, and naming the base
    // interface would under-declare what callers actually get.
    interface.add_method("scope").return_type(
        ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_STORAGE_SCOPE_INTERFACE_NAME,
        )))
        .allow_null(),
    );

    // An instance method, as upstream declares it. `Storage::current()` must be declared the
    // same way: a static method cannot satisfy an interface's instance method.
    interface
        .add_method("current")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_INTERFACE_NAME,
        ))));

    interface
        .add_method("attach")
        .argument(
            Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                CONTEXT_INTERFACE_NAME,
            ))),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_STORAGE_SCOPE_INTERFACE_NAME,
        ))));

    interface
}
