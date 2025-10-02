use chrono::{DateTime, Utc};
use phper::{
    arrays::ZArray,
    classes::{ClassEntity, StateClass, Visibility},
    functions::ReturnType,
    types::ReturnTypeHint,
};
use std::{
    convert::Infallible,
    sync::Mutex,
};
use once_cell::sync::Lazy;
use opentelemetry_sdk::logs::InMemoryLogExporter;

pub type MemoryExporterClass = StateClass<()>;

const MEMORY_EXPORTER_CLASS_NAME: &str = r"OpenTelemetry\API\Logs\MemoryLogsExporter";

// Static instance of the exporter
pub static MEMORY_EXPORTER: Lazy<Mutex<InMemoryLogExporter>> = Lazy::new(|| {
    Mutex::new(InMemoryLogExporter::default())
});

pub fn make_logs_memory_exporter_class() -> ClassEntity<()> {
    let mut class = ClassEntity::<()>::new(MEMORY_EXPORTER_CLASS_NAME);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class.add_static_method("count", Visibility::Public, |_| {
        let exporter = MEMORY_EXPORTER.lock().unwrap();
        let logs_count = exporter.get_emitted_logs().map(|logs| logs.len()).unwrap_or(0);
        Ok::<_, Infallible>(logs_count as i64)
    })
        .return_type(ReturnType::new(ReturnTypeHint::Int));

    class.add_static_method("getLogs", Visibility::Public, |_| {
        let mut result = ZArray::new();
        let exporter = MEMORY_EXPORTER.lock().unwrap();
        let logs = exporter.get_emitted_logs().unwrap_or_default();
        for (_i, log) in logs.iter().enumerate() {
            let mut arr = ZArray::new();
            // Log body
            arr.insert("body", format!("{:?}", log.record.body()));
            // Severity number
            let severity_number = log.record.severity_number().map(|s| s as i64).unwrap_or(0);
            arr.insert("severity_number", severity_number);
            // Severity text
            let severity_text = log.record.severity_text().unwrap_or("");
            arr.insert("severity_text", severity_text);
            arr.insert(
                "event_name",
                log.record.event_name().unwrap_or(""),
            );
            // Trace context
            if let Some(trace_context) = log.record.trace_context() {
                arr.insert("trace_id", format!("{:032x}", trace_context.trace_id));
                arr.insert("span_id", format!("{:016x}", trace_context.span_id));
                if let Some(flags) = trace_context.trace_flags {
                    arr.insert("trace_flags", flags.to_u8() as i64);
                }
            }
            // Timestamp
            let timestamp = log
                .record
                .timestamp()
                .and_then(|ts| ts.duration_since(std::time::UNIX_EPOCH).ok())
                .and_then(|d| {
                    let secs = d.as_secs() as i64;
                    let nsecs = d.subsec_nanos();
                    DateTime::<Utc>::from_timestamp(secs, nsecs).map(|dt| dt.to_rfc3339())
                });
            if let Some(ts) = timestamp {
                arr.insert("timestamp", ts);
            }

            // Observed timestamp
            let observed_timestamp = log
                .record
                .observed_timestamp()
                .and_then(|ts| ts.duration_since(std::time::UNIX_EPOCH).ok())
                .and_then(|d| {
                    let secs = d.as_secs() as i64;
                    let nsecs = d.subsec_nanos();
                    DateTime::<Utc>::from_timestamp(secs, nsecs).map(|dt| dt.to_rfc3339())
                });
            if let Some(ts) = observed_timestamp {
                arr.insert("observed_timestamp", ts);
            }
            // Attributes
            let mut attributes = ZArray::new();
            for (key, value) in log.record.attributes_iter() {
                attributes.insert(key.as_str(), format!("{:?}", value));
            }
            arr.insert("attributes", attributes);
            // Instrumentation scope
            let mut scope = ZArray::new();
            scope.insert("name", log.instrumentation.name());
            scope.insert("version", log.instrumentation.version().as_deref().unwrap_or(""));
            scope.insert("schema_url", log.instrumentation.schema_url().as_deref().unwrap_or(""));
            // Scope attributes
            let mut scope_attributes = ZArray::new();
            for kv in log.instrumentation.attributes() {
                scope_attributes.insert(kv.key.as_str(), format!("{:?}", kv.value));
            }
            scope.insert("attributes", scope_attributes);

            arr.insert("instrumentation_scope", scope);
            // Resource (sorted by key)
            let mut resource = ZArray::new();
            let mut resource_kv: Vec<_> = log.resource.iter().collect();
            resource_kv.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
            for (k, v) in resource_kv {
                resource.insert(k.as_str(), format!("{:?}", v));
            }
            arr.insert("resource", resource);
            result.insert((), arr);

        }
        Ok::<_, Infallible>(result)
    })
        .return_type(ReturnType::new(ReturnTypeHint::Array));

    class.add_static_method("reset", Visibility::Public, |_| {
        let exporter = MEMORY_EXPORTER.lock().unwrap();
        exporter.reset();
        Ok::<_, Infallible>(())
    })
        .return_type(ReturnType::new(ReturnTypeHint::Void));

    class
}