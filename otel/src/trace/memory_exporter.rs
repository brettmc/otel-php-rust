use crate::util;
use opentelemetry_sdk::trace::InMemorySpanExporter;
use phper::arrays::ZArray;
use phper::classes::{ClassEntity, StateClass, Visibility};
use phper::functions::ReturnType;
use phper::types::ReturnTypeHint;
use std::convert::Infallible;
use std::sync::{LazyLock, Mutex};

pub type MemoryExporterClass = StateClass<()>;

const MEMORY_EXPORTER_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\SpanExporter\Memory";

// Static instance of the exporter
pub static MEMORY_EXPORTER: LazyLock<Mutex<InMemorySpanExporter>> =
    LazyLock::new(|| Mutex::new(InMemorySpanExporter::default()));

pub fn make_memory_exporter_class() -> ClassEntity<()> {
    let mut class = ClassEntity::<()>::new(MEMORY_EXPORTER_CLASS_NAME);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_static_method("count", Visibility::Public, |_| {
            let exporter = MEMORY_EXPORTER.lock().unwrap();
            let spans_count = exporter
                .get_finished_spans()
                .map(|spans| spans.len())
                .unwrap_or(0);
            Ok::<_, Infallible>(spans_count as i64)
        })
        .return_type(ReturnType::new(ReturnTypeHint::Int));

    class
        .add_static_method("getSpans", Visibility::Public, |_| {
            let mut result = ZArray::new();
            let exporter = MEMORY_EXPORTER.lock().unwrap();
            let spans = exporter.get_finished_spans().unwrap_or_default();
            for span in spans.iter() {
                let mut arr = ZArray::new();
                arr.insert("name", &*span.name);
                let mut span_context = ZArray::new();
                span_context.insert("trace_id", span.span_context.trace_id().to_string());
                span_context.insert("span_id", span.span_context.span_id().to_string());
                span_context.insert(
                    "trace_flags",
                    format!("{:02x}", span.span_context.trace_flags()),
                );
                span_context.insert("is_remote", span.span_context.is_remote());
                arr.insert("span_context", span_context);
                arr.insert("parent_span_id", span.parent_span_id.to_string());
                arr.insert("span_kind", format!("{:?}", span.span_kind));
                let start_time = span
                    .start_time
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_micros();
                arr.insert("start_time", start_time as i64);
                let end_time = span
                    .end_time
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_micros();
                arr.insert("end_time", end_time as i64);
                let mut scope = ZArray::new();
                scope.insert("name", span.instrumentation_scope.name());
                scope.insert(
                    "version",
                    span.instrumentation_scope.version().unwrap_or(""),
                );
                scope.insert(
                    "schema_url",
                    span.instrumentation_scope.schema_url().unwrap_or(""),
                );
                scope.insert(
                    "attributes",
                    util::attributes_to_zarray(span.instrumentation_scope.attributes()),
                );
                arr.insert("instrumentation_scope", scope);
                arr.insert("status", format!("{:?}", span.status));
                arr.insert(
                    "attributes",
                    util::attributes_to_zarray(span.attributes.iter()),
                );
                let mut events = ZArray::new();
                for event in span.events.clone() {
                    let mut event_arr = ZArray::new();
                    event_arr.insert("name", &*event.name);
                    let timestamp = event
                        .timestamp
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_micros();
                    event_arr.insert("timestamp", timestamp as i64);

                    event_arr.insert(
                        "attributes",
                        util::attributes_to_zarray(event.attributes.iter()),
                    );
                    events.insert((), event_arr);
                }
                arr.insert("events", events);
                // Add span links
                let mut links = ZArray::new();
                for link in span.links.clone() {
                    let mut link_arr = ZArray::new();
                    let mut link_context = ZArray::new();
                    link_context.insert("trace_id", link.span_context.trace_id().to_string());
                    link_context.insert("span_id", link.span_context.span_id().to_string());
                    link_context.insert(
                        "trace_flags",
                        format!("{:02x}", link.span_context.trace_flags()),
                    );
                    link_context.insert("is_remote", link.span_context.is_remote());
                    link_arr.insert("span_context", link_context);

                    // Link attributes
                    link_arr.insert(
                        "attributes",
                        util::attributes_to_zarray(link.attributes.iter()),
                    );
                    links.insert((), link_arr);
                }
                arr.insert("links", links);
                //add to final result set
                result.insert((), arr);
            }
            Ok::<_, Infallible>(result)
        })
        .return_type(ReturnType::new(ReturnTypeHint::Array));

    class
        .add_static_method("reset", Visibility::Public, |_| {
            let exporter = MEMORY_EXPORTER.lock().unwrap();
            exporter.reset();
            Ok::<_, Infallible>(())
        })
        .return_type(ReturnType::new(ReturnTypeHint::Void));

    class
}
