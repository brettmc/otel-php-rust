use phper::{
    classes::InterfaceEntity,
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};

pub const PROPAGATION_SETTER_INTERFACE_NAME: &str =
    r"OpenTelemetry\Context\Propagation\PropagationSetterInterface";

/// Writes propagation headers into a carrier of the caller's choosing.
///
/// `$carrier` is by reference and untyped, as upstream: by reference so an array carrier can
/// be modified in place, untyped so an object carrier is equally acceptable.
pub fn make_propagation_setter_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(PROPAGATION_SETTER_INTERFACE_NAME);

    interface
        .add_method("set")
        .argument(Argument::new("carrier").by_ref())
        .argument(Argument::new("key").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("value").with_type_hint(ArgumentTypeHint::String))
        .return_type(ReturnType::new(ReturnTypeHint::Void));

    interface
}
