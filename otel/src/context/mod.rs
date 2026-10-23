// `context::context` mirrors the PHP type it defines: OpenTelemetry\Context\Context.
// Every other module here is named after its PHP type too, so renaming this one to
// silence the lint would make it the only module that doesn't.
#[allow(clippy::module_inception)]
pub mod context;
pub mod context_interface;
pub mod context_key;
pub mod context_key_interface;
pub mod context_storage_interface;
pub mod context_storage_scope_interface;
pub mod implicit_context_keyed_interface;
pub mod propagation;
pub mod scope;
pub mod scope_interface;
pub mod storage;
