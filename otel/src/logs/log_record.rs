use phper::classes::ClassEntity;
use opentelemetry::{
    KeyValue,
    logs::{Severity, AnyValue},
    trace::{TraceId, SpanId, TraceFlags},
    Context,
    trace::TraceContextExt,
};
use phper::{
    alloc::ToRefOwned,
    classes::{StateClass, Visibility},
    functions::Argument,
    types::ArgumentTypeHint,
    values::ZVal,
};
use std::time::SystemTime;

pub const LOG_RECORD_CLASS_NAME: &str = r"OpenTelemetry\API\Logs\LogRecord";

// State holds the values to be set on the builder later
#[derive(Default)]
pub struct LogRecordState {
    pub body: Option<AnyValue>,
    pub severity: Option<Severity>,
    pub severity_text: Option<String>,
    pub attributes: Vec<KeyValue>,
    pub event_name: Option<String>,
    pub timestamp: Option<SystemTime>,
    pub trace_id: Option<TraceId>,
    pub span_id: Option<SpanId>,
    pub trace_flags: Option<TraceFlags>,
}

pub type LogRecordClass = StateClass<LogRecordState>;

/// Map a severity number from the log data model onto its `Severity`.
///
/// The numbers are the spec's own 1-24 scale, so they are a wire format rather than an
/// internal detail. A number outside that range is a userland mistake we cannot represent;
/// `Info` keeps the record rather than dropping it.
pub(crate) fn map_severity_number(severity: u8) -> Severity {
    match severity {
        1 => Severity::Trace,
        2 => Severity::Trace2,
        3 => Severity::Trace3,
        4 => Severity::Trace4,
        5 => Severity::Debug,
        6 => Severity::Debug2,
        7 => Severity::Debug3,
        8 => Severity::Debug4,
        9 => Severity::Info,
        10 => Severity::Info2,
        11 => Severity::Info3,
        12 => Severity::Info4,
        13 => Severity::Warn,
        14 => Severity::Warn2,
        15 => Severity::Warn3,
        16 => Severity::Warn4,
        17 => Severity::Error,
        18 => Severity::Error2,
        19 => Severity::Error3,
        20 => Severity::Error4,
        21 => Severity::Fatal,
        22 => Severity::Fatal2,
        23 => Severity::Fatal3,
        24 => Severity::Fatal4,
        _ => Severity::Info,
    }
}

fn zval_to_value(zval: &phper::values::ZVal) -> Option<opentelemetry::Value> {
    use opentelemetry::{Array, Value};
    if let Ok(s) = zval.expect_z_str() {
        Some(Value::String(s.to_str().ok()?.to_owned().into()))
    } else if let Ok(i) = zval.expect_long() {
        Some(Value::I64(i))
    } else if let Ok(f) = zval.expect_double() {
        Some(Value::F64(f))
    } else if let Ok(b) = zval.expect_bool() {
        Some(Value::Bool(b))
    } else if let Ok(arr) = zval.expect_z_arr() {
        // Try to collect as homogeneous array
        let mut bools = Vec::new();
        let mut i64s = Vec::new();
        let mut f64s = Vec::new();
        let mut strings = Vec::new();
        let mut value_type = None;

        for (_, v) in arr.iter() {
            if let Ok(b) = v.expect_bool()
                && (value_type.is_none() || value_type == Some("bool")) {
                bools.push(b);
                value_type = Some("bool");
                continue;
            }
            if let Ok(i) = v.expect_long()
                && (value_type.is_none() || value_type == Some("i64")) {
                i64s.push(i);
                value_type = Some("i64");
                continue;
            }
            if let Ok(f) = v.expect_double()
                && (value_type.is_none() || value_type == Some("f64")) {
                f64s.push(f);
                value_type = Some("f64");
                continue;
            }
            if let Ok(s) = v.expect_z_str()
                && (value_type.is_none() || value_type == Some("string")) {
                strings.push(s.to_str().unwrap_or("").to_owned().into());
                value_type = Some("string");
                continue;
            }
            // If we get here, it's a mixed or unsupported type
            tracing::warn!("Unsupported or mixed array element type in attribute array, skipping attribute: {:?}", v);
            return None;
        }

        let array_value = match value_type {
            Some("bool") => Value::Array(Array::Bool(bools)),
            Some("i64") => Value::Array(Array::I64(i64s)),
            Some("f64") => Value::Array(Array::F64(f64s)),
            Some("string") => Value::Array(Array::String(strings)),
            _ => {
                tracing::warn!("Empty or unsupported array for attribute, skipping.");
                return None;
            }
        };
        Some(array_value)
    } else {
        None
    }
}

fn insert_attribute(state: &mut LogRecordState, key: String, value_zval: &phper::values::ZVal) -> Result<(), phper::Error> {
    let value = match zval_to_value(value_zval) {
        Some(val) => val,
        None => {
            tracing::warn!("Unsupported attribute value type for key '{}', skipping: {:?}", key, value_zval);
            return Ok(());
        }
    };
    state.attributes.push(KeyValue::new(key, value));
    Ok(())
}

/// Convert a PHP value to a log record body.
///
/// Shared by `__construct` and `setBody` so the two cannot disagree about what a body may be.
fn zval_to_any_value(zval: &ZVal) -> phper::Result<AnyValue> {
    Ok(if let Some(s) = zval.as_z_str() {
        // Convert to owned String immediately to avoid lifetime issues
        AnyValue::String(s.to_str()?.to_owned().into())
    } else if let Some(b) = zval.as_bool() {
        // Checked before long: a PHP bool converts to a long, so testing long first would
        // record `true` as 1.
        AnyValue::Boolean(b)
    } else if let Some(i) = zval.as_long() {
        AnyValue::Int(i)
    } else if let Some(f) = zval.as_double() {
        AnyValue::Double(f)
    } else {
        // fallback: string representation using Debug
        AnyValue::String(format!("{:?}", zval).into())
    })
}

pub fn make_log_record_class() -> ClassEntity<LogRecordState> {
    let mut class =
        ClassEntity::<LogRecordState>::new_with_default_state_constructor(LOG_RECORD_CLASS_NAME);

    class
        .add_method("__construct", Visibility::Public, |this, arguments| {
            // `mixed $body = null` upstream, so the slot may be absent or hold any type.
            if let Some(body_zval) = arguments.first()
                && body_zval.as_null().is_none()
            {
                this.as_mut_state().body = Some(zval_to_any_value(body_zval)?);
            }
            // Set trace context from current OpenTelemetry context if available
            let ctx = Context::current();
            let span = ctx.span();
            let span_ctx = span.span_context();
            if span_ctx.is_valid() {
                this.as_mut_state().trace_id = Some(span_ctx.trace_id());
                this.as_mut_state().span_id = Some(span_ctx.span_id());
                this.as_mut_state().trace_flags = Some(span_ctx.trace_flags());
            }
            Ok::<_, phper::Error>(())
        })
        // `with_default_value` takes PHP *source text*, not a value: it is stored verbatim
        // and compiled as a constant expression when something reads the default. So `"null"`
        // here means the expression `null`, and a *string* default would need its quotes
        // inside the Rust literal (`"''"`, not `""`). A value that is not a parseable
        // expression compiles to a malformed AST and segfaults on the first read -- which
        // reflection does, so `ReflectionClass` over the extension is enough to trigger it.
        // Every default declared in this extension is read by declared-defaults-readable.phpt.
        .argument(
            Argument::new("body")
                .with_type_hint(ArgumentTypeHint::Mixed)
                .with_default_value("null"),
        );

    class
        .add_method("setTimestamp", Visibility::Public, |this, arguments| {
            // Accept nanoseconds since UNIX_EPOCH as int
            let ts_zval = &arguments[0];
            let nanos: u64 = ts_zval.expect_long()? as u64;
            let system_time = std::time::UNIX_EPOCH
                + std::time::Duration::from_secs(nanos / 1_000_000_000)
                + std::time::Duration::from_nanos(nanos % 1_000_000_000);
            tracing::debug!("Setting LogRecord timestamp to {:?}", system_time);
            this.as_mut_state().timestamp = Some(system_time);
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("timestamp").with_type_hint(ArgumentTypeHint::Int));

    class
        .add_method("setSeverityNumber", Visibility::Public, |this, arguments| {
            let severity = arguments[0].expect_long()? as u8;
            this.as_mut_state().severity = Some(map_severity_number(severity));
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("severityNumber").with_type_hint(ArgumentTypeHint::Int));

    class
        .add_method("setBody", Visibility::Public, |this, arguments| {
            // `mixed $body = null` upstream, so the slot may be absent.
            let body = match arguments.first() {
                Some(zval) if zval.as_null().is_none() => Some(zval_to_any_value(zval)?),
                _ => None,
            };
            this.as_mut_state().body = body;
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(
            Argument::new("body")
                .with_type_hint(ArgumentTypeHint::Mixed)
                .with_default_value("null"),
        );

    class
        .add_method("setSeverityText", Visibility::Public, |this, arguments| {
            let text = arguments[0].expect_z_str()?.to_str()?.to_owned();
            this.as_mut_state().severity_text = Some(text);

            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("severityText").with_type_hint(ArgumentTypeHint::String));

    class
        .add_method("setAttribute", Visibility::Public, |this, arguments| {
            let state = this.as_mut_state();
            let key = arguments[0].expect_z_str()?.to_str()?.to_string();
            let value_zval = &arguments[1];
            insert_attribute(state, key, value_zval)?;
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        // Both required, as upstream has them. An `.optional()` on `$value` would let
        // `setAttribute('k')` past Zend's arity check and into a slot the handler indexes,
        // panicking across the FFI boundary -- which aborts the process rather than throwing.
        .argument(Argument::new("name").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("value").with_type_hint(ArgumentTypeHint::Mixed));

    class
        .add_method("setAttributes", Visibility::Public, |this, arguments| {
            let state = this.as_mut_state();
            let attrs_zval = &arguments[0];
            let attrs_arr = attrs_zval.expect_z_arr()?;
            for (k, v) in attrs_arr.iter() {
                // Support both string and integer keys
                let key = match k {
                    phper::arrays::IterKey::ZStr(zs) => zs.to_str()?.to_string(),
                    phper::arrays::IterKey::Index(i) => i.to_string(),
                };
                insert_attribute(state, key, v)?;
            }
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::Iterable));

    class
        .add_method("setEventName", Visibility::Public, |this, arguments| {
            let name = arguments[0].expect_z_str()?.to_str()?.to_owned();
            this.as_mut_state().event_name = Some(name);
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("eventName").with_type_hint(ArgumentTypeHint::String));

    class
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_spec_severity_number_maps_to_its_own_severity() {
        // The mapping has to be injective over 1-24: two numbers collapsing onto one
        // `Severity` would silently relabel a record's severity.
        let mapped: Vec<Severity> = (1..=24).map(map_severity_number).collect();
        for (i, severity) in mapped.iter().enumerate() {
            assert_eq!(
                *severity as u8,
                (i + 1) as u8,
                "severity number {} mapped to {:?}",
                i + 1,
                severity
            );
        }
    }

    #[test]
    fn the_range_boundaries_map_to_the_extremes() {
        assert_eq!(map_severity_number(1), Severity::Trace);
        assert_eq!(map_severity_number(24), Severity::Fatal4);
    }

    #[test]
    fn numbers_outside_the_spec_range_fall_back_to_info() {
        // 0 and anything above 24 are not severities in the data model; the record is kept
        // at Info rather than dropped.
        assert_eq!(map_severity_number(0), Severity::Info);
        assert_eq!(map_severity_number(25), Severity::Info);
        assert_eq!(map_severity_number(u8::MAX), Severity::Info);
    }
}
