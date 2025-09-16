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
use phper::classes::ClassEntry;
use phper::functions::ZFunc;

/// Determines if a function should be traced based on the provided targets.
pub fn should_trace(func: &ZFunc, targets: &[(Option<String>, String)]) -> bool {
    let name_zstr = func.get_function_or_method_name();
    let function_name = match name_zstr.to_str() {
        Ok(name) => name,
        Err(_) => return false,
    };

    let mut parts = function_name.splitn(2, "::");
    let class_part = parts.next();
    let method_part = parts.next();

    let observed_name_pair = if let Some(method) = method_part {
        (class_part.map(|s| s.to_string()), method.to_string())
    } else {
        (None, function_name.to_string())
    };

    if targets.iter().any(|target| target.0 == observed_name_pair.0 && target.1 == observed_name_pair.1) {
        return true;
    }

    if observed_name_pair.0.is_none() {
        return false;
    }

    let ce = match func.get_class() {
        Some(class_entry) => class_entry,
        None => return false,
    };
    for (target_class_name, target_method_name) in targets.iter() {
        if let Some(interface_name) = target_class_name {
            if target_method_name == &observed_name_pair.1 {
                if let Ok(iface_ce) = ClassEntry::from_globals(interface_name.clone()) {
                    if ce.is_instance_of(&iface_ce) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

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