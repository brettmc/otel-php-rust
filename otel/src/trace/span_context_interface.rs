use phper::{
    classes::InterfaceEntity,
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};

pub const SPAN_CONTEXT_INTERFACE_NAME: &str = r"OpenTelemetry\API\Trace\SpanContextInterface";

pub fn make_span_context_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(SPAN_CONTEXT_INTERFACE_NAME);

    // Upstream's factories take `?TraceStateInterface $traceState = null` as a fourth
    // argument. There is no TraceState class in this extension to name, and the SDK's trace
    // state is not plumbed through either, so the argument is omitted rather than declared
    // with a type that does not exist.
    interface
        .add_static_method("create")
        .argument(Argument::new("traceId").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("spanId").with_type_hint(ArgumentTypeHint::String))
        .argument(
            Argument::new("traceFlags")
                .with_type_hint(ArgumentTypeHint::Int)
                .with_default_value("0"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_CONTEXT_INTERFACE_NAME,
        ))));

    interface
        .add_static_method("createFromRemoteParent")
        .argument(Argument::new("traceId").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("spanId").with_type_hint(ArgumentTypeHint::String))
        .argument(
            Argument::new("traceFlags")
                .with_type_hint(ArgumentTypeHint::Int)
                .with_default_value("0"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_CONTEXT_INTERFACE_NAME,
        ))));

    interface
        .add_static_method("getInvalid")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_CONTEXT_INTERFACE_NAME,
        ))));

    for getter in ["getTraceId", "getSpanId"] {
        interface
            .add_method(getter)
            .return_type(ReturnType::new(ReturnTypeHint::String));
    }

    interface
        .add_method("getTraceFlags")
        .return_type(ReturnType::new(ReturnTypeHint::Int));

    for predicate in ["isValid", "isRemote", "isSampled"] {
        interface
            .add_method(predicate)
            .return_type(ReturnType::new(ReturnTypeHint::Bool));
    }

    // Deliberately omitted, because the class does not implement them and declaring them
    // here would make `SpanContext` fail to satisfy its own interface at registration:
    // `getTraceIdBinary()`, `getSpanIdBinary()` and `getTraceState()`.

    interface
}
