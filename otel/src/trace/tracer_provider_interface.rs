use phper::{
    classes::{InterfaceEntity},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use crate::trace::tracer_interface::TRACER_INTERFACE_NAME;

pub const TRACER_PROVIDER_INTERFACE_NAME: &str = r"OpenTelemetry\API\Trace\TracerProviderInterface";

pub fn make_tracer_provider_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(TRACER_PROVIDER_INTERFACE_NAME);
    interface
        .add_method("getTracer")
        .argument(Argument::new("name").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("version").with_type_hint(ArgumentTypeHint::String).allow_null().with_default_value("null"))
        .argument(Argument::new("schemaUrl").with_type_hint(ArgumentTypeHint::String).allow_null().with_default_value("null"))
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::Iterable).with_default_value("[]"))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(TRACER_INTERFACE_NAME))));

    interface
}
