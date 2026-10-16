use phper::{
    classes::{InterfaceEntity},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use crate::context::context_interface::CONTEXT_INTERFACE_NAME;

pub const TEXT_MAP_PROPAGATOR_INTERFACE_NAME: &str = r"OpenTelemetry\Context\Propagation\TextMapPropagatorInterface";

pub fn make_text_map_propagator_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(TEXT_MAP_PROPAGATOR_INTERFACE_NAME);

    interface
        .add_method("inject")
        .argument(Argument::new("carrier").with_type_hint(ArgumentTypeHint::Mixed).by_ref())
        .argument(Argument::new("setter").allow_null().with_default_value("null"))
        .argument(Argument::new("context")
            .with_type_hint(ArgumentTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME)))
            .with_default_value("null")
            .allow_null()
        )
        .return_type(ReturnType::new(ReturnTypeHint::Void));

    interface
        .add_method("extract")
        .argument(Argument::new("carrier"))
        .argument(Argument::new("getter").allow_null().with_default_value("null"))
        .argument(Argument::new("context")
            .with_type_hint(ArgumentTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME)))
            .with_default_value("null")
            .allow_null()
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME))));

    interface
}
