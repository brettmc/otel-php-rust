use phper::classes::InterfaceEntity;

pub fn make_status_code_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(r"OpenTelemetry\API\Trace\StatusCode");

    interface.add_constant("STATUS_UNSET", "Unset");
    interface.add_constant("STATUS_OK", "Ok");
    interface.add_constant("STATUS_ERROR", "Error");

    interface
}
