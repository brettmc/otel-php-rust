use phper::{
    classes::{
        ClassEntity,
        StateClass,
        Visibility,
    },
};
use std::{
    cell::RefCell,
    convert::Infallible,
};
use opentelemetry::Context;
use opentelemetry::ContextGuard;

const SCOPE_CLASS_NAME: &str = "OpenTelemetry\\API\\Context\\Scope";
pub type ScopeClass = StateClass<Option<Context>>;

thread_local! {
    static ACTIVE_CONTEXT_GUARD: RefCell<Option<ContextGuard>> = RefCell::new(None);
}

fn store_context_guard(guard: ContextGuard) {
    ACTIVE_CONTEXT_GUARD.with(|slot| {
        *slot.borrow_mut() = Some(guard);
    });
}

/// Retrieves and removes the context guard from thread-local storage.
fn take_context_guard() -> Option<ContextGuard> {
    ACTIVE_CONTEXT_GUARD.with(|slot| slot.borrow_mut().take())
}

pub fn make_scope_class() -> ClassEntity<Option<Context>> {
    let mut class =
        ClassEntity::<Option<Context>>::new_with_default_state_constructor(SCOPE_CLASS_NAME);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("detach", Visibility::Public, |_this, _| -> phper::Result<()> {
            //TODO drop the active context guard here
            Ok(())
        });

    class
}
