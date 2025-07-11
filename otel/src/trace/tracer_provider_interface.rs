use phper::{
    classes::{InterfaceEntity},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};

pub fn make_tracer_provider_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(r"OpenTelemetry\API\Trace\TracerProviderInterface");
    interface
        .add_method("getTracer")
        .argument(Argument::new("name").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("version").optional().with_type_hint(ArgumentTypeHint::String).allow_null())
        .argument(Argument::new("schemaUrl").optional().with_type_hint(ArgumentTypeHint::String).allow_null())
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::ClassEntry(String::from("Iterable"))).with_default_value("[]"))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\API\Trace\TracerInterface"))));

    interface
}
