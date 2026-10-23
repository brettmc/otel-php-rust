use crate::config;
use chrono::Utc;
use phper::ini::ini_get;
use std::{
    collections::HashSet,
    ffi::CStr,
    fmt::{self, Write},
    fs::{File, OpenOptions},
    io::Write as _,
    process,
    sync::{LazyLock, Mutex, OnceLock},
    thread,
};
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::{Layer, Registry, filter::LevelFilter, layer::Context, prelude::*};

/// Where log lines go. Resolved once from `otel.log_file`, so the configured file is opened
/// once for the life of the process instead of once per line.
enum LogSink {
    Stdout,
    Stderr,
    /// Opened with `append`, so concurrent writes from forked workers sharing this
    /// descriptor each seek to the end rather than overwriting one another.
    File(Mutex<File>),
    /// The configured file could not be opened. The reason was reported when we resolved
    /// the sink; retrying the open for every subsequent line would just repeat it.
    Unavailable,
}

static LOG_SINK: OnceLock<LogSink> = OnceLock::new();
static LOGGER_PIDS: LazyLock<Mutex<HashSet<u32>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

/// Initialize logging subscriber if it's not already running for this PID (for SAPIs that
/// spawn worker processes)
pub fn init_once() {
    let pid = process::id();
    {
        let mut logger_pids = LOGGER_PIDS.lock().unwrap();
        if !logger_pids.insert(pid) {
            return;
        }
    }

    let log_level = ini_get::<Option<&CStr>>(config::ini::OTEL_LOG_LEVEL)
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

    let subscriber = Registry::default()
        .with(PhpErrorLogLayer)
        .with(level_filter);
    // Emitted outside the LOGGER_PIDS lock: the layer writes to the log sink, and holding an
    // unrelated lock across that is a deadlock waiting to be introduced.
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(()) => tracing::debug!("Logging::initialized level={}", level_filter),
        Err(_) => tracing::debug!("Logging::already initialized for pid={}", pid),
    }
}

fn resolve_log_sink() -> LogSink {
    let log_file = ini_get::<Option<&CStr>>(config::ini::OTEL_LOG_FILE)
        .and_then(|cstr| cstr.to_str().ok())
        .unwrap_or("/dev/stderr");

    match log_file {
        "/dev/stdout" => LogSink::Stdout,
        "/dev/stderr" => LogSink::Stderr,
        path => match OpenOptions::new().create(true).append(true).open(path) {
            Ok(file) => LogSink::File(Mutex::new(file)),
            Err(err) => {
                eprintln!("[ERROR] Failed to open log file '{}': {}", path, err);
                LogSink::Unavailable
            }
        },
    }
}

fn log_message(message: &str) {
    match LOG_SINK.get_or_init(resolve_log_sink) {
        LogSink::Stdout => println!("{}", message),
        LogSink::Stderr => eprintln!("{}", message),
        LogSink::File(file) => {
            // A poisoned lock means a previous writer panicked mid-write; the handle is
            // still usable for appending, so keep logging rather than panicking in turn.
            let mut file = match file.lock() {
                Ok(file) => file,
                Err(poisoned) => poisoned.into_inner(),
            };
            if let Err(err) = writeln!(file, "{}", message) {
                eprintln!("[ERROR] Failed to write to log file: {}", err);
            }
        }
        LogSink::Unavailable => {}
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
        let mut visitor = LogVisitor {
            message: String::new(),
        };
        event.record(&mut visitor);
        message.push_str(&visitor.message);

        log_message(message.as_str());
    }
}
