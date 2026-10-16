use phper::{
    classes::{InterfaceEntity},
};

pub const SPAN_INTERFACE_NAME: &str = r"OpenTelemetry\API\Trace\SpanInterface";

pub fn make_span_interface() -> InterfaceEntity {
    

    InterfaceEntity::new(SPAN_INTERFACE_NAME)
}
