use phper::classes::ClassEntity;
use opentelemetry::logs::{Severity, AnyValue};
use phper::{
    alloc::ToRefOwned,
    classes::{StateClass, Visibility},
};

pub const LOG_RECORD_CLASS_NAME: &str = r"OpenTelemetry\API\Logs\LogRecord";

// State holds the values to be set on the builder later
#[derive(Default)]
pub struct LogRecordState {
    pub body: Option<AnyValue>,
    pub severity: Option<Severity>,
}

pub type LogRecordClass = StateClass<LogRecordState>;

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
            Ok::<_, phper::Error>(())
        })
        .argument(phper::functions::Argument::new("body").optional().with_default_value("").with_type_hint(phper::types::ArgumentTypeHint::String));

    class.add_method("setSeverityNumber", Visibility::Public, |this, arguments| {
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
    .argument(phper::functions::Argument::new("severityNumber"));

    class.add_method("setBody", Visibility::Public, |this, arguments| {
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
}
