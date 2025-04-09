use phper::{
    classes::{InterfaceEntity},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};

pub fn make_text_map_propagator_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(r"OpenTelemetry\Context\Propagation\TextMapPropagatorInterface");

    interface
        .add_method("inject")
        .argument(Argument::new("carrier").with_type_hint(ArgumentTypeHint::Mixed).by_ref())
        .argument(Argument::new("setter").allow_null().with_default_value("null"))
        .argument(Argument::new("context")
            .with_type_hint(ArgumentTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ContextInterface")))
            .with_default_value("null")
            .allow_null()
        )
        .return_type(ReturnType::new(ReturnTypeHint::Void));

    interface
        .add_method("extract")
        .argument(Argument::new("carrier"))
        .argument(Argument::new("getter").allow_null().with_default_value("null"))
        .argument(Argument::new("context")
            .with_type_hint(ArgumentTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ContextInterface")))
            .with_default_value("null")
            .allow_null()
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ContextInterface"))));

    interface
}
