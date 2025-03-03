use phper::ini::{ini_get};
use phper::sys::{php_error_docref, E_WARNING, E_NOTICE};
use tracing::{Event, Level, Subscriber, field::{Visit, Field}};
use tracing_subscriber::{layer::Context, Layer, filter::LevelFilter, Registry, prelude::*};
use std::ffi::{CString, CStr};
use std::fmt::{self, Write};

pub fn init() {
    let log_level = ini_get::<Option<&CStr>>("otel.log_level")
        .and_then(|cstr| cstr.to_str().ok())
        .unwrap_or("none");
    let level_filter = match log_level {
        "error" => LevelFilter::ERROR,
        "warn" => LevelFilter::WARN,
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        _ => LevelFilter::OFF,
    };
    let subscriber = Registry::default().with(PhpErrorLogLayer).with(level_filter);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

/// A visitor that captures structured log fields into a string.
struct LogVisitor {
    message: String,
}

impl Visit for LogVisitor {
    /// Handles string values in structured logging
    fn record_str(&mut self, field: &Field, value: &str) {
        write!(&mut self.message, " {}={}", field.name(), value).ok();
    }

    /// Handles non-string values (e.g., integers, structs) by formatting them as debug output
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        write!(&mut self.message, " {}={:?}", field.name(), value).ok();
    }
}

/// A custom tracing layer that sends logs to the PHP error log.
pub struct PhpErrorLogLayer;

impl<S> Layer<S> for PhpErrorLogLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut message = format!("{}: {}", event.metadata().target(), event.metadata().name());

        // Capture structured log fields
        let mut visitor = LogVisitor { message: String::new() };
        event.record(&mut visitor);
        message.push_str(&visitor.message);

        // Convert Rust string to C string for PHP
        let c_message = CString::new(message).unwrap_or_else(|_| CString::new("Log message error").unwrap());

        unsafe {
            let error_type = match *event.metadata().level() {
                Level::ERROR => E_WARNING,
                Level::WARN => E_WARNING,
                _ => E_NOTICE, // INFO and DEBUG as NOTICE
            };

            // Send to PHP error log
            php_error_docref(std::ptr::null(), error_type.try_into().unwrap(), c_message.as_ptr());
        }
    }
}
