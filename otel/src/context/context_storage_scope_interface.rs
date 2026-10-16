use phper::{
    classes::{Interface, InterfaceEntity},
    functions::ReturnType,
    types::ReturnTypeHint,
};
use crate::context::context_interface::CONTEXT_INTERFACE_NAME;

pub const CONTEXT_STORAGE_SCOPE_INTERFACE_NAME: &str =
    r"OpenTelemetry\Context\ContextStorageScopeInterface";

/// A scope that also exposes the context it was attached for, and a per-scope scratch map.
///
/// Upstream splits this from `ScopeInterface` on purpose: `ScopeInterface` is the minimum a
/// caller needs to end a scope (`detach()`), while only a scope handed out by a *storage* is
/// required to be able to say which context it belongs to. Declaring `context()` on the
/// base interface would promise it of every scope.
///
/// The `ArrayAccess` half is a scratch map keyed by string, which instrumentation uses to
/// hang per-scope bookkeeping off a scope without needing somewhere else to put it.
pub fn make_context_storage_scope_interface(scope_interface: &Interface) -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(CONTEXT_STORAGE_SCOPE_INTERFACE_NAME);

    interface.extends(scope_interface.clone());
    // `extends` may be called more than once, which is what lets this mirror upstream's
    // `extends ScopeInterface, ArrayAccess`. ArrayAccess is an SPL interface, reached by name
    // rather than by a handle we registered.
    interface.extends(Interface::from_name("ArrayAccess"));

    interface
        .add_method("context")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_INTERFACE_NAME,
        ))));

    // Upstream re-declares only `offsetSet`, to narrow `$offset` to string in its docblock;
    // the other three come from ArrayAccess itself. Re-declaring it here would mean pinning
    // argument names that must then match ArrayAccess's own, so it is left inherited --
    // an implementation satisfies the same contract either way.

    interface
}
