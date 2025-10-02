use phper::{
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use std::convert::Infallible;
use opentelemetry::logs::{
    Logger,
    LogRecord,
};
use opentelemetry_sdk::logs::SdkLogger;
use crate::logs::log_record::{LOG_RECORD_CLASS_NAME, LogRecordState};

pub type LoggerClass = StateClass<Option<SdkLogger>>;

const LOGGER_CLASS_NAME: &str = r"OpenTelemetry\API\Logs\Logger";

pub fn make_logger_class(
    logger_interface: Interface,
) -> ClassEntity<Option<SdkLogger>> {
    let mut class =
        ClassEntity::<Option<SdkLogger>>::new_with_default_state_constructor(LOGGER_CLASS_NAME);

    class.implements(logger_interface);
    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("emit", Visibility::Public, |this, arguments| {
            tracing::debug!("Logger::emit called");
            // Get the logger
            let logger: &SdkLogger = this.as_state().as_ref().unwrap();

            // Get the LogRecordClass object from PHP
            let record_zval = &arguments[0];
            let record_obj = record_zval.expect_z_obj();
            // SAFETY: Only safe if the object is a LogRecordClass (guaranteed by PHP type hint)
            let record_state = unsafe { record_obj?.as_state_obj::<LogRecordState>().as_state() };

            // Build the log record using the logger's builder
            let mut log_record = logger.create_log_record();

            if let Some(severity) = record_state.severity {
                log_record.set_severity_number(severity);
            }
            if let Some(ref body) = record_state.body {
                log_record.set_body(body.clone());
            }
            if let Some(ref severity_text) = record_state.severity_text {
                //TODO avoid leaking memory here (Cow<'static, str>)
                let static_str: &'static str = Box::leak(severity_text.clone().into_boxed_str());
                log_record.set_severity_text(static_str);
            }
            if let Some(ref event_name) = record_state.event_name {
                //TODO avoid leaking memory here (Cow<'static, str>)
                let static_str: &'static str = Box::leak(event_name.clone().into_boxed_str());
                log_record.set_event_name(static_str);
            }
            if let (Some(trace_id), Some(span_id)) = (record_state.trace_id, record_state.span_id) {
                log_record.set_trace_context(
                    trace_id,
                    span_id,
                    record_state.trace_flags,
                );
            }
            if let Some(timestamp) = record_state.timestamp {
                log_record.set_timestamp(timestamp);
            }
            if !record_state.attributes.is_empty() {
                for attr in &record_state.attributes {
                    let any_value = match &attr.value {
                        opentelemetry::Value::String(s) => opentelemetry::logs::AnyValue::String(s.clone()),
                        opentelemetry::Value::Bool(b) => opentelemetry::logs::AnyValue::Boolean(*b),
                        opentelemetry::Value::I64(i) => opentelemetry::logs::AnyValue::Int(*i),
                        opentelemetry::Value::F64(f) => opentelemetry::logs::AnyValue::Double(*f),
                        _ => opentelemetry::logs::AnyValue::String(format!("{:?}", attr.value).into()),
                    };
                    log_record.add_attribute(attr.key.clone(), any_value);
                }
            }
            tracing::debug!("Built LogRecord: {:?}", log_record);

            logger.emit(log_record);

            Ok::<_, phper::Error>(())
        })
        .argument(Argument::new("record").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(LOG_RECORD_CLASS_NAME))))
        .return_type(ReturnType::new(ReturnTypeHint::Void));

    class
}
