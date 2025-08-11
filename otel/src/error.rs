use std::fmt;
use opentelemetry::KeyValue;
use phper::objects::ZObj;

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


//if let Some(mut exception_obj) = exception {
//                         tracing::error!("Auto::Laminas::pre (MvcEvent::completeRequest) - exception found: {:?}", exception_obj);
//                         //rather than record_error, which is limited, use span_ref.add_event with extra attributes
//                         let attributes = vec![
//                             KeyValue::new("exception.message", exception_obj.call("getMessage", [])
//                                 .ok()
//                                 .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())))
//                                 .unwrap_or_else(|| "Unknown exception".to_string())
//                             ),
//                             KeyValue::new("exception.type", exception_obj.get_class().get_name().to_str().unwrap_or("Unknown").to_owned()),
//                             KeyValue::new("exception.stacktrace", exception_obj.call("getTraceAsString", [])
//                                 .ok()
//                                 .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())))
//                                 .unwrap_or_else(|| "No stack trace".to_string())),
//                         ];
//                         span_ref.add_event("exception", attributes);
//                     }