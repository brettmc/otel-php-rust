use std::sync::OnceLock;
use tokio::runtime::{Builder, Runtime};

static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// The tokio runtime required to construct the gRPC (tonic) OTLP exporters.
///
/// Returns `None` if the runtime could not be built, so a failure here degrades to
/// "no OTLP export" rather than taking down the PHP process.
///
/// This must be a multi-threaded runtime even though we only ever `block_on` exporter
/// construction here. The tonic exporter keeps using this runtime for every export, driven
/// from the batch processor's own thread, which never calls `block_on`; on a
/// `new_current_thread` runtime nothing then polls the connection and the processor's
/// shutdown flush times out (`tests/otlp/export_*_grpc.phpt` catch exactly this).
pub fn init_tokio_runtime() -> Option<&'static Runtime> {
    if let Some(runtime) = TOKIO_RUNTIME.get() {
        return Some(runtime);
    }
    match Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => {
            // `set` can still lose a race with another thread; either way a runtime is
            // then present, so use whichever won.
            let _ = TOKIO_RUNTIME.set(runtime);
            TOKIO_RUNTIME.get()
        }
        Err(err) => {
            tracing::error!(
                "Failed to create Tokio runtime required for gRPC export: {}",
                err
            );
            None
        }
    }
}
