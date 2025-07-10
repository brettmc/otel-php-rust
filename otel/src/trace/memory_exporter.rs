use phper::{
    classes::{ClassEntity, StateClass, Visibility},
};
use phper::functions::ReturnType;
use phper::types::ReturnTypeHint;
use phper::arrays::ZArray;
use std::sync::Mutex;
use std::convert::Infallible;
use once_cell::sync::Lazy;
use opentelemetry_sdk::trace::InMemorySpanExporter;

pub type MemoryExporterClass = StateClass<()>;

const MEMORY_EXPORTER_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\SpanExporter\Memory";

// Static instance of the exporter
pub static MEMORY_EXPORTER: Lazy<Mutex<InMemorySpanExporter>> = Lazy::new(|| {
    Mutex::new(InMemorySpanExporter::default())
});

pub fn make_memory_exporter_class() -> ClassEntity<()> {
    let mut class = ClassEntity::<()>::new(MEMORY_EXPORTER_CLASS_NAME);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class.add_static_method("count", Visibility::Public, |_| {
        let exporter = MEMORY_EXPORTER.lock().unwrap();
        let spans_count = exporter.get_finished_spans().map(|spans| spans.len()).unwrap_or(0);
        Ok::<_, Infallible>(spans_count as i64)
    })
        .return_type(ReturnType::new(ReturnTypeHint::Int));

    class.add_static_method("getSpans", Visibility::Public, |_| {
        let mut result = ZArray::new();
        let exporter = MEMORY_EXPORTER.lock().unwrap();
        let spans = exporter.get_finished_spans().unwrap_or_default();
        for (_i, span) in spans.iter().enumerate() {
            let mut arr = ZArray::new();
            arr.insert("name", &*span.name);
            let mut span_context = ZArray::new();
            span_context.insert("trace_id", span.span_context.trace_id().to_string());
            span_context.insert("span_id", span.span_context.span_id().to_string());
            span_context.insert("trace_flags", format!("{:02x}", span.span_context.trace_flags()));
            span_context.insert("is_remote", span.span_context.is_remote());
            arr.insert("span_context", span_context);
            arr.insert("parent_span_id", span.parent_span_id.to_string());
            arr.insert("span_kind", format!("{:?}", span.span_kind));
            let start_time = span.start_time.duration_since(std::time::UNIX_EPOCH).unwrap().as_micros();
            arr.insert("start_time", start_time as i64);
            let end_time = span.end_time.duration_since(std::time::UNIX_EPOCH).unwrap().as_micros();
            arr.insert("end_time", end_time as i64);
            let mut scope = ZArray::new();
            scope.insert("name", &*span.instrumentation_scope.name());
            scope.insert("version", span.instrumentation_scope.version().as_deref().unwrap_or(""));
            scope.insert("schema_url", span.instrumentation_scope.schema_url().as_deref().unwrap_or(""));
            let mut scope_attributes = ZArray::new();
            for kv in span.instrumentation_scope.attributes() {
                match &kv.value {
                    opentelemetry::Value::String(s) => scope_attributes.insert(kv.key.as_str(), s.as_str()),
                    opentelemetry::Value::I64(i) => scope_attributes.insert(kv.key.as_str(), *i),
                    opentelemetry::Value::F64(f) => scope_attributes.insert(kv.key.as_str(), *f),
                    opentelemetry::Value::Bool(b) => scope_attributes.insert(kv.key.as_str(), *b),
                    _ => {
                        // For simplicity, we will not handle other types like bool, array, etc.
                        continue;
                    }
                }
            }
            scope.insert("attributes", scope_attributes);
            arr.insert("instrumentation_scope", scope);
            arr.insert("status", format!("{:?}", span.status));
            let mut attributes = ZArray::new();
            for kv in span.attributes.iter() {
                match &kv.value {
                    opentelemetry::Value::String(s) => attributes.insert(kv.key.as_str(), s.as_str()),
                    opentelemetry::Value::I64(i) => attributes.insert(kv.key.as_str(), *i),
                    opentelemetry::Value::F64(f) => attributes.insert(kv.key.as_str(), *f),
                    opentelemetry::Value::Bool(b) => attributes.insert(kv.key.as_str(), *b),
                    opentelemetry::Value::Array(arr) => {
                        let mut arr_values = ZArray::new();
                        match arr {
                            opentelemetry::Array::Bool(vec) => {
                                for v in vec {
                                    arr_values.insert((), *v);
                                }
                            }
                            opentelemetry::Array::I64(vec) => {
                                for v in vec {
                                    arr_values.insert((), *v);
                                }
                            }
                            opentelemetry::Array::F64(vec) => {
                                for v in vec {
                                    arr_values.insert((), *v);
                                }
                            }
                            opentelemetry::Array::String(vec) => {
                                for v in vec {
                                    arr_values.insert((), v.as_str());
                                }
                            }
                            _ => {
                                // For simplicity, we will not handle other types like bool, array, etc.
                                continue;
                            }
                        }
                        attributes.insert(kv.key.as_str(), arr_values);
                    }
                    _ => {
                        // For simplicity, we will not handle other types like bool, array, etc.
                        continue;
                    }
                }
            }
            arr.insert("attributes", attributes);
            let mut events = ZArray::new();
            for event in span.events.clone() {
                let mut event_arr = ZArray::new();
                event_arr.insert("name", &*event.name);
                let timestamp = event.timestamp.duration_since(std::time::UNIX_EPOCH).unwrap().as_micros();
                event_arr.insert("timestamp", timestamp as i64);

                let mut event_attributes = ZArray::new();
                for kv in event.attributes.iter() {
                    match &kv.value {
                        opentelemetry::Value::String(s) => event_attributes.insert(kv.key.as_str(), s.as_str()),
                        opentelemetry::Value::I64(i) => event_attributes.insert(kv.key.as_str(), *i),
                        opentelemetry::Value::F64(f) => event_attributes.insert(kv.key.as_str(), *f),
                        opentelemetry::Value::Bool(b) => event_attributes.insert(kv.key.as_str(), *b),
                        opentelemetry::Value::Array(arr) => {
                            let mut arr_values = ZArray::new();
                            match arr {
                                opentelemetry::Array::Bool(vec) => for v in vec { arr_values.insert((), *v); },
                                opentelemetry::Array::I64(vec) => for v in vec { arr_values.insert((), *v); },
                                opentelemetry::Array::F64(vec) => for v in vec { arr_values.insert((), *v); },
                                opentelemetry::Array::String(vec) => for v in vec { arr_values.insert((), v.as_str()); },
                                _ => continue,
                            }
                            event_attributes.insert(kv.key.as_str(), arr_values);
                        }
                        _ => continue,
                    }
                }
                event_arr.insert("attributes", event_attributes);
                events.insert((), event_arr);
            }
            arr.insert("events", events);
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