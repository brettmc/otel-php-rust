use phper::classes::InterfaceEntity;

pub const SPAN_KIND_INTERFACE_NAME: &str = r"OpenTelemetry\API\Trace\SpanKind";

/// The span kinds accepted by `SpanBuilderInterface::setSpanKind()`.
///
/// Values match upstream `OpenTelemetry\API\Trace\SpanKind` exactly — they travel over the
/// wire and appear in exported spans, so they are part of the contract rather than an
/// internal encoding.
pub fn make_span_kind_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(SPAN_KIND_INTERFACE_NAME);

    interface.add_constant("KIND_INTERNAL", 0i64);
    interface.add_constant("KIND_CLIENT", 1i64);
    interface.add_constant("KIND_SERVER", 2i64);
    interface.add_constant("KIND_PRODUCER", 3i64);
    interface.add_constant("KIND_CONSUMER", 4i64);

    interface
}
