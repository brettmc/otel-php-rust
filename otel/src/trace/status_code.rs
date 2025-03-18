use phper::{
    classes::{ClassEntity, Visibility},
};

// TODO: phper interfaces+classes do not support constants.
// Related [PR](https://github.com/phper-framework/phper/pull/171)
const STATUS_CODE_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\StatusCode";

pub fn make_status_code_class() -> ClassEntity<()> {
    let mut class =
        ClassEntity::new(STATUS_CODE_CLASS_NAME);

    class.add_static_property("STATUS_UNSET", Visibility::Public, "Unset");
    class.add_static_property("STATUS_OK", Visibility::Public, "Ok");
    class.add_static_property("STATUS_ERROR", Visibility::Public, "Error");

    class
}
