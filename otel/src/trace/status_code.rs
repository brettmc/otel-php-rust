use phper::{
    classes::{ClassEntity, Visibility},
};

// TODO: phper interfaces+classes do not support constants
const STATUS_CODE_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\StatusCode";

pub fn make_status_code_class() -> ClassEntity<()> {
    let mut class =
        ClassEntity::new(STATUS_CODE_CLASS_NAME);

    class.add_static_property("STATUS_UNSET", Visibility::Public, "Unset");
    class.add_static_property("STATUS_OK", Visibility::Public, "Ok");
    class.add_static_property("STATUS_ERROR", Visibility::Public, "Error");

    class
}
