use phper::{
    classes::{InterfaceEntity},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use crate::logs::logger_interface::LOGGER_INTERFACE_NAME;

pub const LOGGER_PROVIDER_INTERFACE_NAME: &str = r"OpenTelemetry\API\Logs\LoggerProviderInterface";

pub fn make_logger_provider_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(LOGGER_PROVIDER_INTERFACE_NAME);
    interface
        .add_method("getLogger")
        .argument(Argument::new("name").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("version").optional().with_type_hint(ArgumentTypeHint::String).allow_null())
        .argument(Argument::new("schemaUrl").optional().with_type_hint(ArgumentTypeHint::String).allow_null())
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::Iterable).with_default_value("[]"))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(LOGGER_INTERFACE_NAME))));

    interface
}
