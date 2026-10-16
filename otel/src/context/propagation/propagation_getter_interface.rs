use phper::{
    classes::InterfaceEntity,
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};

pub const PROPAGATION_GETTER_INTERFACE_NAME: &str =
    r"OpenTelemetry\Context\Propagation\PropagationGetterInterface";

/// Reads propagation headers out of a carrier of the caller's choosing.
///
/// `$carrier` is deliberately untyped, here as upstream: it may be an array, a PSR-7 request,
/// or anything the implementation understands. Typing it would narrow the interface below
/// what upstream accepts.
pub fn make_propagation_getter_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(PROPAGATION_GETTER_INTERFACE_NAME);

    interface
        .add_method("keys")
        .argument(Argument::new("carrier"))
        .return_type(ReturnType::new(ReturnTypeHint::Array));

    interface
        .add_method("get")
        .argument(Argument::new("carrier"))
        .argument(Argument::new("key").with_type_hint(ArgumentTypeHint::String))
        .return_type(ReturnType::new(ReturnTypeHint::String).allow_null());

    interface
}
