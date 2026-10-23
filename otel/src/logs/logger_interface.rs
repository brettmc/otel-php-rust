use crate::logs::log_record::LOG_RECORD_CLASS_NAME;
use phper::{
    classes::InterfaceEntity,
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};

pub const LOGGER_INTERFACE_NAME: &str = r"OpenTelemetry\API\Logs\LoggerInterface";

pub fn make_logger_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(LOGGER_INTERFACE_NAME);

    interface
        .add_method("emit")
        .argument(
            Argument::new("record").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                LOG_RECORD_CLASS_NAME,
            ))),
        )
        .return_type(ReturnType::new(ReturnTypeHint::Void));

    interface
}
