use std::fmt;
use opentelemetry::KeyValue;
use phper::{
    objects::ZObj,
    values::ZVal,
};

/// A custom error type that wraps a String and implements the Display, Debug, and Error traits.
/// Used for recording string-based errors on a Span.
pub struct StringError(pub String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

/// Convert a PHP exception to a vector of KeyValue attributes for OpenTelemetry.
pub fn php_exception_to_attributes(exception: &mut ZObj) -> Vec<KeyValue> {
    let mut attributes = vec![];
    if let Some(message) = exception.call("getMessage", [])
        .ok()
        .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())))
    {
        attributes.push(KeyValue::new("exception.message", message));
    }
    let exception_type = exception.get_class().get_name().to_str().unwrap_or("Unknown").to_owned();
    attributes.push(KeyValue::new("exception.type", exception_type));
    if let Some(stack_trace) = exception.call("getTraceAsString", [])
        .ok()
        .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())))
    {
        attributes.push(KeyValue::new("exception.stacktrace", stack_trace));
    }
    attributes
}

/// convert the result of error_get_last() to a vector of KeyValue attributes
pub fn php_error_to_attributes(error: &ZVal) -> Vec<KeyValue> {
    let mut attributes = vec![];
    if let Some(arr) = (*error).as_z_arr() {
        attributes.push(KeyValue::new("exception.type", "PHP fatal error".to_owned()));
        // message
        if let Some(msg) = arr.get("message")
            .and_then(|zv| zv.as_z_str())
            .and_then(|s| s.to_str().ok())
        {
            attributes.push(KeyValue::new("exception.message", msg.to_owned()));
        }
        if let (Some(file), Some(line)) = (
            arr.get("file")
                .and_then(|zv| zv.as_z_str())
                .and_then(|s| s.to_str().ok()),
            arr.get("line")
                .and_then(|zv| zv.as_long())
        ) {
            let stacktrace = format!("{}:{}", file, line);
            attributes.push(KeyValue::new("exception.stacktrace", stacktrace));
        }
    }
    attributes
}