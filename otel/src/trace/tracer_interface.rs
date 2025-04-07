use phper::{
    classes::{InterfaceEntity},
    functions::{Argument},
    types::{ArgumentTypeHint},
};

pub fn make_tracer_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(r"OpenTelemetry\API\Trace\TracerInterface");
    interface
        .add_method("spanBuilder")
        .argument(Argument::new("spanName").with_type_hint(ArgumentTypeHint::String))
        //.return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\API\Trace\SpanBuilderInterface"))))
    ;

    interface
}

//public function spanBuilder(string $spanName): SpanBuilderInterface;