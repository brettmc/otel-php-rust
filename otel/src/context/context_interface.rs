use crate::context::{
    context_key_interface::CONTEXT_KEY_INTERFACE_NAME,
    implicit_context_keyed_interface::IMPLICIT_CONTEXT_KEYED_INTERFACE_NAME,
    scope_interface::SCOPE_INTERFACE_NAME,
};
use phper::{
    classes::InterfaceEntity,
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};

pub const CONTEXT_INTERFACE_NAME: &str = r"OpenTelemetry\Context\ContextInterface";

/// Upstream's full `ContextInterface`: all six members.
///
/// Declaring every one matters because this interface, not the concrete class, is what the
/// signatures in this extension hand out. A member the `Context` class implements but this
/// interface omits is unreachable to a caller holding the declared type -- it works at
/// runtime but fails static analysis, which is where userland meets it first.
pub fn make_context_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(CONTEXT_INTERFACE_NAME);

    interface
        .add_static_method("createKey")
        .argument(Argument::new("key").with_type_hint(ArgumentTypeHint::String))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_KEY_INTERFACE_NAME,
        ))));

    interface
        .add_static_method("getCurrent")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_INTERFACE_NAME,
        ))));

    interface
        .add_method("activate")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SCOPE_INTERFACE_NAME,
        ))));

    interface
        .add_method("with")
        .argument(
            Argument::new("key").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                CONTEXT_KEY_INTERFACE_NAME,
            ))),
        )
        // Untyped upstream, declared `mixed` here. PHP treats the two as the same type for
        // inheritance, so an implementer is unaffected either way, and `mixed` is what the
        // value genuinely is.
        .argument(Argument::new("value").with_type_hint(ArgumentTypeHint::Mixed))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_INTERFACE_NAME,
        ))));

    interface
        .add_method("withContextValue")
        .argument(
            Argument::new("value").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                IMPLICIT_CONTEXT_KEYED_INTERFACE_NAME,
            ))),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_INTERFACE_NAME,
        ))));

    // No return type, as upstream. This is not an omission: a return type may only be
    // *narrowed* by an implementer, so declaring `mixed` here would force every
    // implementation to declare `mixed` too -- and a class written against upstream's
    // untyped `get()` would be a fatal incompatible-declaration error against ours. The
    // concrete `Context` class still declares `mixed`, which narrowing permits.
    interface
        .add_method("get")
        .argument(
            Argument::new("key").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                CONTEXT_KEY_INTERFACE_NAME,
            ))),
        );

    interface
}
