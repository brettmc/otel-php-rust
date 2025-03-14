use phper::ini::{ini_get};
use tracing::{Event, Subscriber, field::{Visit, Field}};
use tracing_subscriber::{layer::Context, Layer, filter::LevelFilter, Registry, prelude::*};
use std::collections::HashMap;
use std::ffi::{CStr};
use std::fmt::{self, Write};
use std::fs::OpenOptions;
use std::io::Write as _;
use std::sync::{LazyLock, Mutex, OnceLock};
use std::process;
use std::thread;
use chrono::Utc;

static LOG_FILE_PATH: OnceLock<String> = OnceLock::new();
static LOGGER_PIDS: LazyLock<Mutex<HashMap<u32, ()>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Initialize logging subscriber if it's not already running for this PID for SAPIs that
/// spawn worker processes
pub fn init_once() {
    print_message("logging::init_once".to_string());
    let pid = process::id();
    let mut logger_pids = LOGGER_PIDS.lock().unwrap();
    if logger_pids.contains_key(&pid) {
        tracing::debug!("logging already initialized for pid: {}", pid);
        return;
    }

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
    tracing::debug!("Logging::initialized level={}", level_filter);
    logger_pids.insert(pid, ());
}

fn log_message(message: &str) {
    let log_file = LOG_FILE_PATH.get_or_init(|| {
        ini_get::<Option<&CStr>>("otel.log.file")
            .and_then(|cstr| cstr.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "/var/log/ext-otel.log".to_string())
    });

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
    {
        let _ = writeln!(file, "{}", message); // Ignore errors to prevent panics
    }
}

/// public message printer, for MINIT (before logging is initialized)
/// TODO: honour log levels!
pub fn print_message(message: String) {
    let thread_id = thread::current().id();
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
    log_message(format!("[{}] [DEBUG] [pid={}] [{:?}] {}", timestamp, process::id(), thread_id, message).as_str());
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
        let pid = process::id();
        let thread_id = thread::current().id();
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let mut message = format!(
            "[{}] [{}] [pid={}] [{:?}] {}: {}",
            timestamp,
            event.metadata().level(),
            pid,
            thread_id,
            event.metadata().target(),
            event.metadata().name()
        );

        // Capture structured log fields
        let mut visitor = LogVisitor { message: String::new() };
        event.record(&mut visitor);
        message.push_str(&visitor.message);

        log_message(message.as_str());
    }
}
