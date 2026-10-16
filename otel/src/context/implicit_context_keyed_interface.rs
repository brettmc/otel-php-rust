use phper::{
    classes::InterfaceEntity,
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use crate::context::{
    context_interface::CONTEXT_INTERFACE_NAME,
    scope_interface::SCOPE_INTERFACE_NAME,
};

pub const IMPLICIT_CONTEXT_KEYED_INTERFACE_NAME: &str =
    r"OpenTelemetry\Context\ImplicitContextKeyedInterface";

/// Something that knows which context key it belongs under, so it can store itself.
///
/// This is what lets `$span->activate()` work without the caller naming a key:
/// `activate()` is defined as `Context::getCurrent()->withContextValue($this)->activate()`,
/// and `withContextValue` delegates straight back to `storeInContext`. Upstream's
/// `SpanInterface` extends this rather than redeclaring the two methods.
pub fn make_implicit_context_keyed_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(IMPLICIT_CONTEXT_KEYED_INTERFACE_NAME);

    interface
        .add_method("activate")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SCOPE_INTERFACE_NAME,
        ))));

    interface
        .add_method("storeInContext")
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(
            String::from(CONTEXT_INTERFACE_NAME),
        )))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_INTERFACE_NAME,
        ))));

    interface
}
