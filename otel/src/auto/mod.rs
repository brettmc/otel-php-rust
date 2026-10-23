#[cfg(otel_observer_not_supported)]
pub mod execute;
pub mod execute_data;
#[cfg(otel_observer_supported)]
pub mod observer;
pub mod plugin;
pub mod plugin_manager;
pub mod utils;

use std::panic::{AssertUnwindSafe, catch_unwind};

/// Run `body`, swallowing any panic, and return `None` if it panicked.
///
/// Both auto-instrumentation backends are entered by the Zend VM across an `extern "C"`
/// boundary, where unwinding is undefined behaviour and there is no PHP frame to throw
/// into. A panic in a hook should cost the user that span, not the whole process, so log it
/// and let the instrumented function run uninstrumented.
pub fn guard<T>(what: &str, body: impl FnOnce() -> T) -> Option<T> {
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(value) => Some(value),
        Err(panic) => {
            let message = panic
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "non-string panic payload".to_string());
            tracing::error!(
                "Auto::panic in {}, skipping instrumentation: {}",
                what,
                message
            );
            None
        }
    }
}
