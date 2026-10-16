use crate::trace::span_builder_interface::SPAN_BUILDER_INTERFACE_NAME;
use phper::{
    classes::InterfaceEntity,
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};

pub const TRACER_INTERFACE_NAME: &str = r"OpenTelemetry\API\Trace\TracerInterface";

pub fn make_tracer_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(TRACER_INTERFACE_NAME);
    interface
        .add_method("spanBuilder")
        .argument(Argument::new("spanName").with_type_hint(ArgumentTypeHint::String))
        // Naming the return type is what makes the rest of a fluent chain checkable:
        // without it, `$tracer->spanBuilder($n)->setParent($c)` is untyped from the first
        // arrow onwards.
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    interface
}
