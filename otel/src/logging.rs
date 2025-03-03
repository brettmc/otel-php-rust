use phper::ini::{ini_get};
use tracing::{Event, Subscriber, field::{Visit, Field}};
use tracing_subscriber::{layer::Context, Layer, filter::LevelFilter, Registry, prelude::*};
use std::ffi::{CStr};
use std::fmt::{self, Write};
use std::fs::OpenOptions;
use std::io::Write as _;
use std::sync::OnceLock;

static LOG_FILE_PATH: OnceLock<String> = OnceLock::new();

pub fn init() {
    let log_file = ini_get::<Option<&CStr>>("otel.log.file")
        .and_then(|cstr| cstr.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "/var/log/ext-otel.log".to_string());
    LOG_FILE_PATH.set(log_file).expect("LOG_FILE_PATH already initialized");

    let log_level = ini_get::<Option<&CStr>>("otel.log.level")
        .and_then(|cstr| cstr.to_str().ok())
        .unwrap_or("none");

    let level_filter = match log_level {
        "error" => LevelFilter::ERROR,
        "warn" => LevelFilter::WARN,
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        "trace" => LevelFilter::TRACE,
        _ => LevelFilter::OFF,
    };

    let subscriber = Registry::default().with(PhpErrorLogLayer).with(level_filter);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

fn log_to_file(message: &str) {
    //let log_file = "/var/log/ext-otel.log";
    let log_file = LOG_FILE_PATH.get().map(|s| s.as_str()).unwrap_or("/var/log/ext-otel.log");

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_file) {
        let _ = writeln!(file, "{}", message); // Ignore errors to prevent panics
    }
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

        log_to_file(message.as_str());
    }
}
