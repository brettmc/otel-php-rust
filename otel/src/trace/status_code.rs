use phper::classes::InterfaceEntity;

pub const STATUS_CODE_INTERFACE_NAME: &str = r"OpenTelemetry\API\Trace\StatusCode";

pub fn make_status_code_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(STATUS_CODE_INTERFACE_NAME);

    interface.add_constant("STATUS_UNSET", "Unset");
    interface.add_constant("STATUS_OK", "Ok");
    interface.add_constant("STATUS_ERROR", "Error");

    interface
}
