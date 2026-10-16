use phper::classes::InterfaceEntity;

pub const CONTEXT_KEY_INTERFACE_NAME: &str = r"OpenTelemetry\Context\ContextKeyInterface";

/// Upstream's `ContextKeyInterface` is a pure marker: no methods, no constants.
///
/// It exists so that `ContextInterface::with()`/`get()` can name a key type, and so that
/// userland can define its own key classes. Keys are compared by object *identity*, never by
/// name, so there is deliberately nothing here for an implementation to get wrong.
pub fn make_context_key_interface() -> InterfaceEntity {
    InterfaceEntity::new(CONTEXT_KEY_INTERFACE_NAME)
}
