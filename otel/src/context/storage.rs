//! Bridges PHP's object lifetime to opentelemetry-rust's `Context`.
//!
//! A `Context` is immutable and cheap to clone, but PHP needs to hand userland a handle it
//! can pass around and compare. That handle is an integer `context_id`, exposed as a private
//! `context_id` property on the PHP object and resolved back to an `Arc<Context>` here.
//! A `context_id` of 0 means "no stored instance" (see [`resolve_context`]), which is what
//! `Context::getCurrent()` reports — so 0 is a normal value, not an error.
//!
//! Three thread-locals, with distinct jobs:
//!
//! - `CONTEXT_STORAGE` — `context_id` -> `Arc<Context>`, the contexts PHP currently holds.
//! - `GUARD_STACK` — the `ContextGuard`s from `activate()`, innermost last. A guard restores
//!   the previous context when dropped, so this must stay a stack: dropping out of order
//!   would restore the wrong context.
//! - `CONTEXT_GUARD_MAP` — guards owned by the auto-instrumentation observer, keyed by
//!   `ExecuteData` address. The observer attaches on function entry and detaches on exit,
//!   which does not nest the way userland `activate()`/`detach()` does, so it cannot share
//!   the stack above.
//!
//! All three are thread-local because a request belongs to a thread: under a threaded SAPI,
//! sharing them would leak one request's active span into another's.
//!
//! Entries outlive both their PHP object and their scope deliberately. An activated context
//! is kept alive by the guard stack after the PHP `Context` object is collected, and a
//! detached `Scope` can still be asked for its context. Because PHP-side holders reference a
//! context by its integer id rather than by `Arc`, `Arc::strong_count` alone cannot decide
//! when an entry is dead — removal is conditional, see [`maybe_remove_context_instance`] and
//! [`detach_context`].

use crate::context::{
    context::{get_instance_id, ContextClassEntity},
    context_interface::CONTEXT_INTERFACE_NAME,
    scope::ScopeClassEntity,
    scope_interface::SCOPE_INTERFACE_NAME,
};
use phper::{
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::{Argument, ReturnType},
    objects::ZObj,
    types::{ArgumentTypeHint, ReturnTypeHint},
    values::{ExecuteData, ZVal},
};
use std::{
    cell::RefCell,
    collections::HashMap,
    convert::Infallible,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use opentelemetry::{
    Context,
    ContextGuard,
    trace::TraceContextExt,
};
use tracing::debug;

const CONTEXT_STORAGE_CLASS_NAME: &str = r"OpenTelemetry\Context\Storage";
pub type StorageClass = StateClass<()>;
pub type StorageClassEntity = ClassEntity<()>;

// When a Context is activated, it is stored in CONTEXT_STORAGE, and a reference to the
// context created and stored as a class property.
thread_local! {
    static CONTEXT_STORAGE: RefCell<HashMap<u64, Arc<Context>>> = RefCell::new(HashMap::new());
    static GUARD_STACK: RefCell<Vec<(ContextGuard, u64)>> = const { RefCell::new(Vec::new()) };
    static CONTEXT_GUARD_MAP: RefCell<HashMap<usize, ContextGuard>> = RefCell::new(HashMap::new()); //for observer use
}
static INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn current_context() -> Arc<Context> {
    current_context_instance_id()
        .and_then(|id| get_context_instance(Some(id)))
        .unwrap_or_else(|| Arc::new(Context::current()))
}

/// Look up a stored context by id, falling back to the current context.
///
/// A userland Context object can outlive its stored instance (for example if its scope was
/// already detached), so a missing id is a recoverable condition rather than a panic: the
/// current context is the same answer `None` gets.
pub fn resolve_context(instance_id: Option<u64>) -> Arc<Context> {
    match instance_id {
        Some(id) => get_context_instance(Some(id)).unwrap_or_else(|| {
            debug!("Context instance {} not found, falling back to current context", id);
            Arc::new(Context::current())
        }),
        None => Arc::new(Context::current()),
    }
}

pub fn get_context_instance(instance_id: Option<u64>) -> Option<Arc<Context>> {
    if let Some(id) = instance_id {
        debug!("Getting context instance {}", id);
        CONTEXT_STORAGE.with(|storage| {
            let maybe_context = storage.borrow().get(&id).cloned();
            if let Some(ref ctx) = maybe_context {
                debug!(
                    "Cloned context instance {} (ref count after clone = {})",
                    id,
                    Arc::strong_count(ctx)
                );
            }
            maybe_context
        })
    } else {
        None
    }
}

pub fn store_context_instance(context: Arc<Context>) -> Option<u64> {
    let instance_id = new_instance_id();
    let count = Arc::strong_count(&context);
    debug!(
        "Storing context instance {} (ref count after clone = {})",
        instance_id, count
    );
    CONTEXT_STORAGE.with(|storage| {
        storage.borrow_mut().insert(instance_id, context)
    });

    Some(instance_id)
}

/// remove context instance if it's not stored in GUARD_STACK
pub fn maybe_remove_context_instance(instance_id: Option<u64>) {
    if let Some(id) = instance_id {
        debug!("Maybe remove context for instance {}", id);
        CONTEXT_STORAGE.with(|storage| {
            let mut map = storage.borrow_mut();
            match map.get(&id) {
                Some(context) => {
                    let count = Arc::strong_count(context);
                    if count == 1 { //the only reference is in CONTEXT_STORAGE
                        debug!(
                            "Removing context instance {} (ref count = 1, no external holders)",
                            id
                        );
                        map.remove(&id);
                    } else {
                        debug!(
                            "Cannot remove context instance {} (ref count = {}, still in use)",
                            id, count
                        );
                    }
                }
                None => {
                    debug!(
                        "Context instance {} not found in CONTEXT_STORAGE, already removed?",
                        id
                    );
                }
            }
        });
    }
}

pub fn remove_context_instance(instance_id: u64) {
    debug!("Removing context instance {}", instance_id);
    CONTEXT_STORAGE.with(|storage| storage.borrow_mut().remove(&instance_id));
}

pub fn attach_context(instance_id: Option<u64>) -> Result<(), &'static str> {
    match instance_id {
        Some(id) => {
            debug!("Attaching context instance {}", id);
            let context = get_context_instance(Some(id)).ok_or("Context not found")?;
            let context_guard = Arc::clone(&context);
            debug!(
                "Before attach: context instance {} has ref count = {}",
                id,
                Arc::strong_count(&context)
            );
            let guard = (*context_guard).clone().attach();
            GUARD_STACK.with(|stack| {
                stack.borrow_mut().push((guard, id));
            });
            Ok(())
        }
        None => Err("No context id provided"),
    }
}

/// Detach a scope, if it is the innermost one.
///
/// Detaching out of order is a userland mistake (the spec calls it undefined behaviour), and
/// the only safe response is to do nothing: a `ContextGuard` restores whatever context was
/// active when it was created, so dropping the inner guard because the *outer* scope was
/// detached would leave the inner span active with no way to ever deactivate it.
///
/// Detaching a context is not the same as deleting it: detaching ends a *scope*, while the
/// context itself lives until nothing can reach it. The canonical post-hook shape reaches
/// the context *after* the detach:
///
/// ```php
/// $scope->detach();
/// Span::fromContext($scope->context())->end();
/// ```
///
/// Both the `Scope` and a `fromContext()` span refer to the context by its integer id, which
/// `Arc::strong_count` cannot see — so a refcount of 1 after popping the guard does not mean
/// nobody is holding it. A context carrying an unended span is therefore kept, and removal
/// left to `Span::end()`, `Context::__destruct`, or the RSHUTDOWN sweep. A context with no
/// span has nothing left to do once detached, so it is released here rather than lingering
/// for the rest of the request.
/// Returns whether the scope was actually detached, so callers can skip the rest of their
/// teardown when the request was refused.
pub fn detach_context(instance_id: Option<u64>) -> bool {
    if let Some(id) = instance_id {
        debug!("Detaching context instance {}", id);
        let detached = GUARD_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            // Inspect the top before popping: popping first and checking afterwards would
            // already have dropped the guard, deactivating a scope nobody asked to detach.
            match stack.last() {
                Some((_, stack_id)) if *stack_id == id => stack.pop().map(|(_, id)| id),
                Some((_, stack_id)) => {
                    debug!(
                        "Not detaching context instance {}, is not top-most (top is {})",
                        id, stack_id
                    );
                    None
                }
                None => {
                    debug!("Not detaching context instance {}, guard stack is empty", id);
                    None
                }
            }
        });
        // Outside the GUARD_STACK borrow: this touches CONTEXT_STORAGE, and the dropped
        // guard above may run arbitrary Drop code.
        if let Some(id) = detached
            && !detached_context_holds_span(id)
        {
            maybe_remove_context_instance(Some(id));
        }
        detached.is_some()
    } else {
        false
    }
}

/// Whether a context carries a span that userland could still want to end.
///
/// An invalid span context means `Context::current()` had no span to begin with, so there is
/// nothing a later `Span::fromContext()` could usefully retrieve.
fn detached_context_holds_span(instance_id: u64) -> bool {
    CONTEXT_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&instance_id)
            .is_some_and(|context| context.span().span_context().is_valid())
    })
}

pub fn current_context_instance_id() -> Option<u64> {
    GUARD_STACK.with(|stack| {
        stack.borrow().last().map(|(_, id)| *id)
    })
}

fn new_instance_id() -> u64 {
    INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub fn new_storage_class() -> StorageClassEntity {
    ClassEntity::<()>::new_with_default_state_constructor(CONTEXT_STORAGE_CLASS_NAME)
}

pub fn build_storage_class(
    class: &mut StorageClassEntity,
    scope_class_entity: &ScopeClassEntity,
    context_class_entity: &ContextClassEntity,
    context_storage_interface: &Interface,
) {
    let _storage_ce = class.bound_class();
    let scope_ce = scope_class_entity.bound_class();
    let context_ce = context_class_entity.bound_class();

    class.implements(context_storage_interface.clone());

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_static_method("current", Visibility::Public, move |_| {
            let context = Arc::new(Context::current());
            let mut object = context_ce.clone().init_object()?;
            *object.as_mut_state() = Some(context);
            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME))));

    let scope_ce_attach = scope_ce.clone();
    class
        .add_method("attach", Visibility::Public, move |_, arguments| {
            let context_obj: &mut ZObj = arguments[0].expect_mut_z_obj()?;
            // `get_instance_id` maps a context_id of 0 to None, which is what
            // `Context::getCurrent()` reports. Reading the property directly turned that 0
            // into `Some(0)`, an id that resolves to no stored context.
            let opt_instance_id = get_instance_id(context_obj);
            attach_context(opt_instance_id).map_err(phper::Error::boxed)?;

            let mut object = scope_ce_attach.init_object()?;
            if let Some(id) = opt_instance_id {
                object.set_property("context_id", id as i64);
            }
            Ok::<_, phper::Error>(object)
        })
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME))))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(SCOPE_INTERFACE_NAME))));

    let scope_ce_scope = scope_ce.clone();
    class
        // Reading the current scope must not change it: `scope()` is an accessor, and the
        // caller detaches through the returned Scope object. Peek at the innermost guard
        // rather than popping it, which would deactivate the context just by looking.
        .add_method("scope", Visibility::Public, move |_, _arguments| {
            let current = GUARD_STACK.with(|stack| {
                stack.borrow().last().map(|(_, context_id)| *context_id)
            });
            match current {
                Some(context_id) => {
                    let mut object = scope_ce_scope.init_object()?;
                    object.set_property("context_id", context_id as i64);
                    Ok::<_, phper::Error>(object.into())
                }
                None => {
                    Ok::<_, phper::Error>(ZVal::from(()))
                }
            }
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(SCOPE_INTERFACE_NAME))).allow_null());
}

pub fn get_context_ids() -> Vec<u64> {
    CONTEXT_STORAGE.with(|cell| {
        let storage = cell.borrow();
        let keys: Vec<u64> = storage.keys().cloned().collect();
        keys
    })
}

pub fn store_guard(exec_data: *mut ExecuteData, guard: ContextGuard) {
    let key = exec_data as *const ExecuteData as usize;
    CONTEXT_GUARD_MAP.with(|map| {
        map.borrow_mut().insert(key, guard);
    });
}

pub fn take_guard(exec_data: *mut ExecuteData) -> Option<ContextGuard> {
    let key = exec_data as *const ExecuteData as usize;
    CONTEXT_GUARD_MAP.with(|map| map.borrow_mut().remove(&key))
}

pub fn clear_context_storage() {
    CONTEXT_STORAGE.with(|storage| storage.borrow_mut().clear());
    GUARD_STACK.with(|stack| stack.borrow_mut().clear());
    CONTEXT_GUARD_MAP.with(|map| map.borrow_mut().clear());
}