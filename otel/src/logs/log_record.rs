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

fn insert_attribute(state: &mut LogRecordState, key: String, value_zval: &phper::values::ZVal) -> Result<(), phper::Error> {
    let any_value = if let Ok(s) = value_zval.expect_z_str() {
        AnyValue::String(s.to_str()?.to_owned().into())
    } else if let Ok(i) = value_zval.expect_long() {
        AnyValue::Int(i)
    } else if let Ok(f) = value_zval.expect_double() {
        AnyValue::Double(f)
    } else if let Ok(b) = value_zval.expect_bool() {
        AnyValue::Boolean(b)
    } else {
        AnyValue::String(format!("{:?}", value_zval).into())
    };
    // Convert AnyValue to opentelemetry::Value
    let value = match any_value {
        AnyValue::String(s) => opentelemetry::Value::String(s),
        AnyValue::Int(i) => opentelemetry::Value::I64(i),
        AnyValue::Double(f) => opentelemetry::Value::F64(f),
        AnyValue::Boolean(b) => opentelemetry::Value::Bool(b),
        _ => todo!(),
    };
    state.attributes.push(KeyValue::new(key, value));
    Ok(())
}

pub fn make_log_record_class() -> ClassEntity<LogRecordState> {
    let mut class =
        ClassEntity::<LogRecordState>::new_with_default_state_constructor(LOG_RECORD_CLASS_NAME);

    class
        .add_method("__construct", Visibility::Public, |this, arguments| {
            //if argument 1 (body) is provided, set it
            if let Some(body_zval) = arguments.get(0) {
                let body_str = body_zval.expect_z_str()?;
                if !body_str.is_empty() {
                    // Convert to owned String immediately to avoid lifetime issues
                    let body_any = AnyValue::String(body_str.to_str()?.to_owned().into());
                    this.as_mut_state().body = Some(body_any);
                }
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
        .argument(phper::functions::Argument::new("body").optional().with_default_value("").with_type_hint(phper::types::ArgumentTypeHint::String));

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
        .argument(phper::functions::Argument::new("timestamp").with_type_hint(ArgumentTypeHint::Int));

    class
        .add_method("setSeverityNumber", Visibility::Public, |this, arguments| {
            let severity = arguments[0].expect_long()? as u8;
            let sev = match severity {
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
            };
            this.as_mut_state().severity = Some(sev);
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(phper::functions::Argument::new("severityNumber").with_type_hint(ArgumentTypeHint::Int));

    class
        .add_method("setBody", Visibility::Public, |this, arguments| {
            let body_zval = &arguments[0];
            let body_any = if let Ok(s) = body_zval.expect_z_str() {
                // Convert to owned String immediately to avoid lifetime issues
                AnyValue::String(s.to_str()?.to_owned().into())
            } else if let Ok(i) = body_zval.expect_long() {
                AnyValue::Int(i)
            } else if let Ok(f) = body_zval.expect_double() {
                AnyValue::Double(f)
            } else if let Ok(b) = body_zval.expect_bool() {
                AnyValue::Boolean(b)
            } else {
                // fallback: string representation using Debug
                let s = format!("{:?}", body_zval);
                AnyValue::String(s.into())
            };
            this.as_mut_state().body = Some(body_any);
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(phper::functions::Argument::new("body"));

    class
        .add_method("setSeverityText", Visibility::Public, |this, arguments| {
            let text = arguments[0].expect_z_str()?.to_str()?.to_owned();
            this.as_mut_state().severity_text = Some(text);

            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(phper::functions::Argument::new("severityText"));

    class
        .add_method("setAttribute", Visibility::Public, |this, arguments| {
            let state = this.as_mut_state();
            let key = arguments[0].expect_z_str()?.to_str()?.to_string();
            let value_zval = &arguments[1];
            insert_attribute(state, key, value_zval)?;
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("key"))
        .argument(Argument::new("value").optional());

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
        .argument(Argument::new("attributes"));

    class
        .add_method("setEventName", Visibility::Public, |this, arguments| {
            let name = arguments[0].expect_z_str()?.to_str()?.to_owned();
            this.as_mut_state().event_name = Some(name);
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(phper::functions::Argument::new("eventName"));

    class
}
