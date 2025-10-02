use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn init_tokio_runtime() -> &'static Runtime {
    if TOKIO_RUNTIME.get().is_none() {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime required for gRPC export");
        TOKIO_RUNTIME.set(runtime).expect("Tokio runtime already set");
    }
    TOKIO_RUNTIME.get().expect("Tokio runtime not initialized")
}
