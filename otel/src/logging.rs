use phper::ini::{ini_get};
use tracing::{Event, Subscriber, field::{Visit, Field}};
use tracing_subscriber::{layer::Context, Layer, filter::LevelFilter, Registry, prelude::*};
use std::collections::HashMap;
use std::ffi::{CStr};
use std::fmt::{self, Write};
use std::fs::OpenOptions;
use std::io::{Write as _};
use std::sync::{LazyLock, Mutex, OnceLock};
use std::process;
use std::thread;
use chrono::Utc;

static LOG_FILE_PATH: OnceLock<String> = OnceLock::new();
static LOGGER_PIDS: LazyLock<Mutex<HashMap<u32, ()>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Initialize logging subscriber if it's not already running for this PID (for SAPIs that
/// spawn worker processes)
pub fn init_once() {
    let pid = process::id();
    let mut logger_pids = LOGGER_PIDS.lock().unwrap();
    if logger_pids.contains_key(&pid) {
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
    let result = tracing::subscriber::set_global_default(subscriber);
    if result.is_ok() {
        logger_pids.insert(pid, ());
        tracing::debug!("Logging::initialized level={}", level_filter);
    } else {
        tracing::debug!("Logging::already initialized for pid={}", pid);
    }
}

fn log_message(message: &str) {
    let log_file = LOG_FILE_PATH.get_or_init(|| {
        ini_get::<Option<&CStr>>("otel.log.file")
            .and_then(|cstr| cstr.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "/dev/stderr".to_string())
    });

    match log_file.as_str() {
        "/dev/stdout" => {
            println!("{}", message);
        }
        "/dev/stderr" => {
            eprintln!("{}", message);
        }
        _ => {
            match OpenOptions::new().create(true).append(true).open(log_file) {
                Ok(mut file) => {
                    if let Err(err) = writeln!(file, "{}", message) {
                        eprintln!("[ERROR] Failed to write to log file '{}': {}", log_file, err);
                    }
                }
                Err(err) => {
                    eprintln!("[ERROR] Failed to open log file '{}': {}", log_file, err);
                }
            }
        }
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
