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

    // Upstream's type is `ContextInterface|false|null`, where null means "use the current
    // context" and false means "no parent, start a new trace". phper has no union type hint,
    // so `false` is unreachable through this signature -- the one place in the extension
    // where a compromise genuinely loses a capability rather than just widening a type:
    // `setParent(false)` is a TypeError, with no spelling that reaches the same behaviour.
    interface
        .add_method("setParent")
        .argument(
            Argument::new("context")
                .with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                    CONTEXT_INTERFACE_NAME,
                )))
                .allow_null(),
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
