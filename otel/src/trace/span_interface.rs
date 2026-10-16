use phper::{
    classes::{Interface, InterfaceEntity},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use crate::{
    context::context_interface::CONTEXT_INTERFACE_NAME,
    trace::span_context_interface::SPAN_CONTEXT_INTERFACE_NAME,
};

pub const SPAN_INTERFACE_NAME: &str = r"OpenTelemetry\API\Trace\SpanInterface";

/// The interface both `Span` and `NonRecordingSpan` implement.
///
/// Declaring the full method set here is what makes the interface load-bearing rather than
/// decorative: the engine rejects an implementation that omits one, so `Span` and
/// `NonRecordingSpan` cannot drift from each other or from upstream unnoticed, and userland
/// can accept a `SpanInterface` parameter and actually call it.
///
/// `activate()` and `storeInContext()` are not declared here, as upstream does not: they come
/// from `ImplicitContextKeyedInterface`, which this extends.
pub fn make_span_interface(
    implicit_context_keyed_interface: &Interface,
) -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(SPAN_INTERFACE_NAME);

    interface.extends(implicit_context_keyed_interface.clone());

    interface
        .add_static_method("fromContext")
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(
            String::from(CONTEXT_INTERFACE_NAME),
        )))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    interface
        .add_static_method("getCurrent")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    interface
        .add_method("getContext")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_CONTEXT_INTERFACE_NAME,
        ))));

    interface
        .add_method("isRecording")
        .return_type(ReturnType::new(ReturnTypeHint::Bool));

    // Upstream types the value `bool|int|float|string|array|null`. phper has no union type
    // hint, so this is `mixed` -- wider than upstream rather than narrower, which keeps every
    // call upstream accepts working.
    interface
        .add_method("setAttribute")
        .argument(Argument::new("key").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("value").with_type_hint(ArgumentTypeHint::Mixed))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    interface
        .add_method("setAttributes")
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::Iterable))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    interface
        .add_method("addLink")
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(
            String::from(SPAN_CONTEXT_INTERFACE_NAME),
        )))
        .argument(
            Argument::new("attributes")
                .with_type_hint(ArgumentTypeHint::Iterable)
                .with_default_value("[]"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    interface
        .add_method("addEvent")
        .argument(Argument::new("name").with_type_hint(ArgumentTypeHint::String))
        .argument(
            Argument::new("attributes")
                .with_type_hint(ArgumentTypeHint::Iterable)
                .with_default_value("[]"),
        )
        .argument(
            Argument::new("timestamp")
                .with_type_hint(ArgumentTypeHint::Int)
                .allow_null()
                .with_default_value("null"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    interface
        .add_method("recordException")
        .argument(Argument::new("exception").with_type_hint(ArgumentTypeHint::ClassEntry(
            String::from("Throwable"),
        )))
        .argument(
            Argument::new("attributes")
                .with_type_hint(ArgumentTypeHint::Iterable)
                .with_default_value("[]"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    interface
        .add_method("updateName")
        .argument(Argument::new("name").with_type_hint(ArgumentTypeHint::String))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    interface
        .add_method("setStatus")
        .argument(Argument::new("code").with_type_hint(ArgumentTypeHint::String))
        .argument(
            Argument::new("description")
                .with_type_hint(ArgumentTypeHint::String)
                .allow_null()
                .with_default_value("null"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    interface
        .add_method("end")
        .argument(
            Argument::new("endEpochNanos")
                .with_type_hint(ArgumentTypeHint::Int)
                .allow_null()
                .with_default_value("null"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::Void));

    // Upstream also declares `getInvalid()` and `wrap(SpanContextInterface)`. Neither exists
    // on `Span` or `NonRecordingSpan` here, so declaring them would make both classes fail
    // to satisfy the interface at registration.

    interface
}
