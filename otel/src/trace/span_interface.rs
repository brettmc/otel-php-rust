use phper::{
    classes::{InterfaceEntity},
};

pub fn make_span_interface() -> InterfaceEntity {
    let interface = InterfaceEntity::new(r"OpenTelemetry\API\Trace\SpanInterface");

    interface
}
