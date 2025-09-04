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
    let select_re = Regex::new(r#"(?si)^\s*(SELECT)\b.*?\bFROM\b(.*)"#).unwrap();
    let insert_re = Regex::new(r#"(?si)^\s*(INSERT)\s+INTO\s+([`"a-zA-Z0-9_\.]+)"#).unwrap();
    let update_re = Regex::new(r#"(?si)^\s*(UPDATE)\s+([`"a-zA-Z0-9_\.]+)"#).unwrap();
    let delete_re = Regex::new(r#"(?si)^\s*(DELETE)\s+FROM\s+([`"a-zA-Z0-9_\.]+)"#).unwrap();

    if let Some(caps) = select_re.captures(sql) {
        let after_from = caps.get(2).map(|m| m.as_str().trim_start());
        if let Some(after_from) = after_from {
            // If FROM is followed by a parenthesis, it's a subquery, not a table
            if after_from.starts_with('(') {
                return None;
            }
            // Otherwise, match the table name
            let table_re = Regex::new(r#"^[`"a-zA-Z0-9_\.]+"#).unwrap();
            if let Some(table_caps) = table_re.captures(after_from) {
                let table = table_caps.get(0).map(|m| m.as_str().replace(['"', '`'], ""));
                return table.map(|t| format!("SELECT {}", t));
            }
        }
    }
    if let Some(caps) = insert_re.captures(sql) {
        let table = caps.get(2).map(|m| m.as_str().replace(['"', '`'], ""));
        return table.map(|t| format!("INSERT {}", t));
    }
    if let Some(caps) = update_re.captures(sql) {
        let table = caps.get(2).map(|m| m.as_str().replace(['"', '`'], ""));
        return table.map(|t| format!("UPDATE {}", t));
    }
    if let Some(caps) = delete_re.captures(sql) {
        let table = caps.get(2).map(|m| m.as_str().replace(['"', '`'], ""));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select() {
        let sql = "SELECT * FROM users";
        assert_eq!(extract_span_name_from_sql(sql), Some("SELECT users".to_string()));
    }

    #[test]
    fn test_insert() {
        let sql = "INSERT INTO products VALUES (1, 'foo')";
        assert_eq!(extract_span_name_from_sql(sql), Some("INSERT products".to_string()));
    }

    #[test]
    fn test_update() {
        let sql = "UPDATE orders SET status = 'shipped'";
        assert_eq!(extract_span_name_from_sql(sql), Some("UPDATE orders".to_string()));
    }

    #[test]
    fn test_delete() {
        let sql = "DELETE FROM sessions WHERE id = 1";
        assert_eq!(extract_span_name_from_sql(sql), Some("DELETE sessions".to_string()));
    }

    #[test]
    fn test_quotes_and_case() {
        let sql = "select * from `MyTable`";
        assert_eq!(extract_span_name_from_sql(sql), Some("SELECT MyTable".to_string()));
        let sql = "INSERT INTO \"OtherTable\"";
        assert_eq!(extract_span_name_from_sql(sql), Some("INSERT OtherTable".to_string()));
    }

    #[test]
    fn test_no_match() {
        let sql = "DROP TABLE users";
        assert_eq!(extract_span_name_from_sql(sql), None);
    }

    #[test]
    fn test_leading_whitespace() {
        let sql = "   SELECT * FROM   users   ";
        assert_eq!(extract_span_name_from_sql(sql), Some("SELECT users".to_string()));
    }

    #[test]
    fn test_multiline_select() {
        let sql = "SELECT\n  *\nFROM\n  users";
        assert_eq!(extract_span_name_from_sql(sql), Some("SELECT users".to_string()));
    }

    #[test]
    fn test_select_distinct() {
        let sql = "select count(distinct(first_name)) * FROM users";
        assert_eq!(extract_span_name_from_sql(sql), Some("SELECT users".to_string()));
    }

    #[test]
    fn test_select_with_subquery_in_from() {
        let sql = "SELECT * FROM (SELECT * FROM users) AS sub";
        // Should not match the subquery, but the outer FROM (which is a subquery, so None)
        assert_eq!(extract_span_name_from_sql(sql), None);
    }

    #[test]
    fn test_select_with_subquery_in_where() {
        let sql = "SELECT * FROM users WHERE id IN (SELECT id FROM orders)";
        // Should match the outer table 'users'
        assert_eq!(extract_span_name_from_sql(sql), Some("SELECT users".to_string()));
    }
}
