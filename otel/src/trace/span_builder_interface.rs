use crate::{
    context::context_interface::CONTEXT_INTERFACE_NAME,
    trace::{
        span_context_interface::SPAN_CONTEXT_INTERFACE_NAME, span_interface::SPAN_INTERFACE_NAME,
    },
};
use phper::{
    classes::InterfaceEntity,
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};

pub const SPAN_BUILDER_INTERFACE_NAME: &str = r"OpenTelemetry\API\Trace\SpanBuilderInterface";

pub fn make_span_builder_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(SPAN_BUILDER_INTERFACE_NAME);

    // Null means "use the current context" and false means "no parent, start a new trace".
    // Before 8.0 phper declares a union untyped, which is what lets `false` through there.
    interface
        .add_method("setParent")
        .argument(
            Argument::new("context").with_type_hint(ArgumentTypeHint::Union(vec![
                ArgumentTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME)),
                ArgumentTypeHint::False,
                ArgumentTypeHint::Null,
            ])),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    interface
        .add_method("addLink")
        .argument(
            Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                SPAN_CONTEXT_INTERFACE_NAME,
            ))),
        )
        .argument(
            Argument::new("attributes")
                .with_type_hint(ArgumentTypeHint::Iterable)
                .with_default_value("[]"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    interface
        .add_method("setAttribute")
        .argument(Argument::new("key").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("value").with_type_hint(ArgumentTypeHint::Mixed))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    interface
        .add_method("setAttributes")
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::Iterable))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    interface
        .add_method("setStartTimestamp")
        .argument(Argument::new("timestampNanos").with_type_hint(ArgumentTypeHint::Int))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    interface
        .add_method("setSpanKind")
        .argument(Argument::new("spanKind").with_type_hint(ArgumentTypeHint::Int))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    interface
        .add_method("startSpan")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    interface
}
