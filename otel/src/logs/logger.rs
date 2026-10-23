use crate::logs::log_record::{LOG_RECORD_CLASS_NAME, LogRecordState};
use opentelemetry::logs::{LogRecord, Logger};
use opentelemetry_sdk::logs::SdkLogger;
use phper::{
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use std::{
    collections::HashSet,
    convert::Infallible,
    sync::{LazyLock, Mutex},
};

pub type LoggerClass = StateClass<Option<SdkLogger>>;

const LOGGER_CLASS_NAME: &str = r"OpenTelemetry\API\Logs\Logger";
static EVENT_NAMES: LazyLock<Mutex<HashSet<&'static str>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// Intern event names to avoid leaking memory since opentelemetry-rust requires &'static str
/// This only reduces memory leaks by re-using event names, but does not eliminate them
fn get_or_intern_event_name(name: &str) -> &'static str {
    let mut set = EVENT_NAMES.lock().unwrap();
    if let Some(&existing) = set.get(name) {
        existing
    } else {
        let leaked: &'static str = Box::leak(name.to_owned().into_boxed_str());
        set.insert(leaked);
        leaked
    }
}

/// Map severity text to static str using opentelemetry::logs::Severity
fn map_severity_text(text: &str) -> Option<&'static str> {
    use opentelemetry::logs::Severity;
    match text {
        "TRACE" => Some(Severity::Trace.name()),
        "TRACE2" => Some(Severity::Trace2.name()),
        "TRACE3" => Some(Severity::Trace3.name()),
        "TRACE4" => Some(Severity::Trace4.name()),
        "DEBUG" => Some(Severity::Debug.name()),
        "DEBUG2" => Some(Severity::Debug2.name()),
        "DEBUG3" => Some(Severity::Debug3.name()),
        "DEBUG4" => Some(Severity::Debug4.name()),
        "INFO" => Some(Severity::Info.name()),
        "INFO2" => Some(Severity::Info2.name()),
        "INFO3" => Some(Severity::Info3.name()),
        "INFO4" => Some(Severity::Info4.name()),
        "WARN" => Some(Severity::Warn.name()),
        "WARN2" => Some(Severity::Warn2.name()),
        "WARN3" => Some(Severity::Warn3.name()),
        "WARN4" => Some(Severity::Warn4.name()),
        "ERROR" => Some(Severity::Error.name()),
        "ERROR2" => Some(Severity::Error2.name()),
        "ERROR3" => Some(Severity::Error3.name()),
        "ERROR4" => Some(Severity::Error4.name()),
        "FATAL" => Some(Severity::Fatal.name()),
        "FATAL2" => Some(Severity::Fatal2.name()),
        "FATAL3" => Some(Severity::Fatal3.name()),
        "FATAL4" => Some(Severity::Fatal4.name()),
        _ => {
            tracing::warn!("Unknown severity text: {}", text);
            None
        }
    }
}

pub fn make_logger_class(logger_interface: Interface) -> ClassEntity<Option<SdkLogger>> {
    let mut class =
        ClassEntity::<Option<SdkLogger>>::new_with_default_state_constructor(LOGGER_CLASS_NAME);

    class.implements(logger_interface);
    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("emit", Visibility::Public, |this, arguments| {
            tracing::debug!("Logger::emit called");
            // Populated only by `LoggerProvider::getLogger()`; anything else is a userland
            // mistake to report, not a reason to panic across the FFI boundary.
            let logger: &SdkLogger = this.as_state().as_ref().ok_or_else(|| {
                phper::Error::boxed("Logger was not created by LoggerProvider::getLogger()")
            })?;

            let record_zval = &arguments[0];
            let record_obj = record_zval.expect_z_obj();
            let record_state = unsafe { record_obj?.as_state_obj::<LogRecordState>().as_state() };

            // Build the log record using the logger's builder
            let mut log_record = logger.create_log_record();

            if let Some(severity) = record_state.severity {
                log_record.set_severity_number(severity);
            }
            if let Some(ref body) = record_state.body {
                log_record.set_body(body.clone());
            }
            if let Some(ref severity_text) = record_state.severity_text
                && let Some(static_str) = map_severity_text(severity_text.as_str())
            {
                log_record.set_severity_text(static_str);
            }
            if let Some(ref event_name) = record_state.event_name {
                let static_str: &'static str = get_or_intern_event_name(event_name.as_str());
                log_record.set_event_name(static_str);
            }
            if let (Some(trace_id), Some(span_id)) = (record_state.trace_id, record_state.span_id) {
                log_record.set_trace_context(trace_id, span_id, record_state.trace_flags);
            }
            if let Some(timestamp) = record_state.timestamp {
                log_record.set_timestamp(timestamp);
            }
            if !record_state.attributes.is_empty() {
                for attr in &record_state.attributes {
                    let any_value = match &attr.value {
                        opentelemetry::Value::String(s) => {
                            opentelemetry::logs::AnyValue::String(s.clone())
                        }
                        opentelemetry::Value::Bool(b) => opentelemetry::logs::AnyValue::Boolean(*b),
                        opentelemetry::Value::I64(i) => opentelemetry::logs::AnyValue::Int(*i),
                        opentelemetry::Value::F64(f) => opentelemetry::logs::AnyValue::Double(*f),
                        _ => opentelemetry::logs::AnyValue::String(
                            format!("{:?}", attr.value).into(),
                        ),
                    };
                    log_record.add_attribute(attr.key.clone(), any_value);
                }
            }
            tracing::debug!("Built LogRecord: {:?}", log_record);

            logger.emit(log_record);

            Ok::<_, phper::Error>(())
        })
        .argument(
            Argument::new("record").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                LOG_RECORD_CLASS_NAME,
            ))),
        )
        .return_type(ReturnType::new(ReturnTypeHint::Void));

    class
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::log_record::map_severity_number;

    #[test]
    fn every_severity_name_is_recognised() {
        // Driven off the number map so the two stay in step: every severity reachable via
        // `setSeverityNumber` must also be nameable via `setSeverityText`.
        for number in 1..=24u8 {
            let name = map_severity_number(number).name();
            assert_eq!(
                map_severity_text(name),
                Some(name),
                "severity text {:?} was not recognised",
                name
            );
        }
    }

    #[test]
    fn the_range_boundaries_are_recognised() {
        assert_eq!(map_severity_text("TRACE"), Some("TRACE"));
        assert_eq!(map_severity_text("FATAL4"), Some("FATAL4"));
    }

    #[test]
    fn unknown_severity_text_is_rejected() {
        // `None` leaves severity_text unset on the record rather than inventing a value.
        assert_eq!(map_severity_text("NOTICE"), None);
        assert_eq!(map_severity_text(""), None);
    }

    #[test]
    fn severity_text_matching_is_case_sensitive() {
        // The data model spells these upper-case; accepting "info" here would make the
        // extension more lenient than the spec, so pin the current behaviour.
        assert_eq!(map_severity_text("info"), None);
        assert_eq!(map_severity_text("Info"), None);
    }
}
