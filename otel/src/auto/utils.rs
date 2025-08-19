use crate::{
    auto::execute_data::get_default_attributes,
    context::storage::store_guard,
};
use opentelemetry::{
    Context,
    KeyValue,
    trace::{
        SpanKind,
        Tracer,
        TraceContextExt,
    }
};
use opentelemetry_sdk::trace::SdkTracer;
use phper::{
    values::ExecuteData,
    objects::ZObj,
};
use regex::Regex;

pub fn extract_span_name_from_sql(sql: &str) -> Option<String> {
    let sql = sql.trim();
    let select_re = Regex::new(r#"(?i)^\s*SELECT.*FROM\s+([`"a-zA-Z0-9_\.]+)"#).unwrap();
    let insert_re = Regex::new(r#"(?i)^\s*INSERT\s+INTO\s+([`"a-zA-Z0-9_\.]+)"#).unwrap();
    let update_re = Regex::new(r#"(?i)^\s*UPDATE\s+([`"a-zA-Z0-9_\.]+)"#).unwrap();
    let delete_re = Regex::new(r#"(?i)^\s*DELETE\s+FROM\s+([`"a-zA-Z0-9_\.]+)"#).unwrap();

    if let Some(caps) = select_re.captures(sql) {
        let table = caps.get(1).map(|m| m.as_str().replace(['"', '`'], ""));
        return table.map(|t| format!("SELECT {}", t));
    }
    if let Some(caps) = insert_re.captures(sql) {
        let table = caps.get(1).map(|m| m.as_str().replace(['"', '`'], ""));
        return table.map(|t| format!("INSERT {}", t));
    }
    if let Some(caps) = update_re.captures(sql) {
        let table = caps.get(1).map(|m| m.as_str().replace(['"', '`'], ""));
        return table.map(|t| format!("UPDATE {}", t));
    }
    if let Some(caps) = delete_re.captures(sql) {
        let table = caps.get(1).map(|m| m.as_str().replace(['"', '`'], ""));
        return table.map(|t| format!("DELETE {}", t));
    }

    None
}

pub fn record_exception(context: &opentelemetry::Context, exception: &mut ZObj) {
    let attributes = crate::error::php_exception_to_attributes(exception);
    context.span().add_event("exception", attributes);
    let message = exception.call("getMessage", [])
        .ok()
        .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())))
        .unwrap_or_default();
    context.span().set_status(opentelemetry::trace::Status::error(message));
}

pub fn start_and_activate_span(
    tracer: SdkTracer,
    span_name: &str,
    attributes: Vec<KeyValue>,
    exec_data: *mut ExecuteData,
    span_kind: SpanKind,
) {
    let mut merged_attributes = get_default_attributes(unsafe { &*exec_data });
    merged_attributes.extend(attributes);
    let span_builder = tracer.span_builder(span_name.to_string())
        .with_kind(span_kind)
        .with_attributes(merged_attributes);
    let span = tracer.build_with_context(span_builder, &Context::current());
    let ctx = Context::current_with_span(span);
    let guard = ctx.attach();
    store_guard(exec_data, guard);
}