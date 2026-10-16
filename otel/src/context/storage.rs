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
//! - `CONTEXT_VALUES` — the PHP-visible key/value pairs of each context, see
//!   [`context_values`] below.
//!
//! All four are thread-local because a request belongs to a thread: under a threaded SAPI,
//! sharing them would leak one request's active span into another's.
//!
//! Entries outlive both their PHP object and their scope deliberately. An activated context
//! is kept alive by the guard stack after the PHP `Context` object is collected, and a
//! detached `Scope` can still be asked for its context. Because PHP-side holders reference a
//! context by its integer id rather than by `Arc`, `Arc::strong_count` alone cannot decide
//! when an entry is dead — removal is conditional, see [`maybe_remove_context_instance`] and
//! [`detach_context`].

use crate::context::{
    context::{get_instance_id, ContextClassEntity, CONTEXT_CLASS_NAME},
    context_interface::CONTEXT_INTERFACE_NAME,
    context_storage_scope_interface::CONTEXT_STORAGE_SCOPE_INTERFACE_NAME,
    scope::ScopeClassEntity,
};
use phper::{
    classes::{ClassEntity, ClassEntry, Interface, StateClass, Visibility},
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
    static CONTEXT_VALUES: RefCell<HashMap<u64, ContextValues>> = RefCell::new(HashMap::new());
}
static INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// The PHP-visible key/value pairs carried by one `Context`.
///
/// The Rust SDK cannot hold these: `Context::with_value<T>` keys by `TypeId::of::<T>()`,
/// resolved at compile time, and a `ContextKey` created at runtime by PHP has no `TypeId`.
/// So the map lives here, beside the `Arc<Context>` it belongs to, and is looked up by the
/// same `context_id`.
///
/// Keys are identified by object handle, mirroring upstream's `spl_object_id($key)`. A handle
/// is reused once its object is freed, so a map that stored only the handle could hand a
/// later, unrelated key the earlier key's value. Upstream guards against this by keeping the
/// key object itself alive (`$self->contextKeys[$id] ??= $key`), and the `key` field below is
/// that same strong reference — it is never read, only held.
#[derive(Default)]
pub struct ContextValues {
    entries: HashMap<u32, ContextValue>,
}

struct ContextValue {
    /// Held so the handle in the map key cannot be recycled onto a different object.
    #[allow(dead_code)]
    key: ZVal,
    value: ZVal,
}

impl ContextValues {
    fn get(&self, handle: u32) -> Option<&ZVal> {
        self.entries.get(&handle).map(|entry| &entry.value)
    }

    /// Return a copy of these pairs with `key` set to `value`, leaving the original untouched.
    ///
    /// `Context::with()` is required to produce a *new* context rather than mutate the
    /// receiver, so the whole map is cloned. `ZVal::clone` is a refcount bump for anything
    /// refcounted, so this is proportional to the number of keys, not to the size of the
    /// values.
    fn with(&self, handle: u32, key: ZVal, value: ZVal) -> Self {
        let mut entries: HashMap<u32, ContextValue> = self
            .entries
            .iter()
            .map(|(h, entry)| {
                (
                    *h,
                    ContextValue {
                        key: entry.key.clone(),
                        value: entry.value.clone(),
                    },
                )
            })
            .collect();
        entries.insert(handle, ContextValue { key, value });
        Self { entries }
    }
}

/// Read one PHP-side value out of a context, by key object handle.
pub fn get_context_value(instance_id: Option<u64>, key_handle: u32) -> Option<ZVal> {
    let id = instance_id?;
    CONTEXT_VALUES.with(|values| {
        values
            .borrow()
            .get(&id)
            .and_then(|map| map.get(key_handle))
            .cloned()
    })
}

/// Store a context whose PHP-side values are `parent`'s plus one more pair.
///
/// Takes the parent's `context_id` rather than its map so that the clone happens under a
/// single borrow, and so a caller cannot accidentally associate a map with the wrong context.
pub fn store_context_with_value(
    context: Arc<Context>,
    parent_instance_id: Option<u64>,
    key_handle: u32,
    key: ZVal,
    value: ZVal,
) -> Option<u64> {
    let instance_id = store_context_instance(context);
    CONTEXT_VALUES.with(|values| {
        let mut map = values.borrow_mut();
        let parent = parent_instance_id.and_then(|id| map.get(&id));
        let merged = match parent {
            Some(parent) => parent.with(key_handle, key, value),
            None => ContextValues::default().with(key_handle, key, value),
        };
        if let Some(id) = instance_id {
            map.insert(id, merged);
        }
    });

    instance_id
}

/// Copy a context's PHP-side values onto another context id.
///
/// `activate()` and `Span::storeInContext()` allocate a fresh `context_id` for a context
/// derived from an existing one. Without this the derived context would silently lose every
/// key its parent carried.
pub fn inherit_context_values(parent_instance_id: Option<u64>, child_instance_id: Option<u64>) {
    let (Some(parent_id), Some(child_id)) = (parent_instance_id, child_instance_id) else {
        return;
    };
    CONTEXT_VALUES.with(|values| {
        let mut map = values.borrow_mut();
        if let Some(parent) = map.get(&parent_id) {
            let copied = ContextValues {
                entries: parent
                    .entries
                    .iter()
                    .map(|(h, entry)| {
                        (
                            *h,
                            ContextValue {
                                key: entry.key.clone(),
                                value: entry.value.clone(),
                            },
                        )
                    })
                    .collect(),
            };
            map.insert(child_id, copied);
        }
    });
}

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
                        remove_context_values(id);
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
    remove_context_values(instance_id);
}

/// Drop a context's PHP-side values, releasing the `ZVal`s it held.
///
/// Called wherever the `Arc<Context>` itself is dropped: the values are only reachable through
/// that `context_id`, so leaving them behind would hold userland objects alive for the rest of
/// the request with nothing able to read them.
fn remove_context_values(instance_id: u64) {
    CONTEXT_VALUES.with(|values| values.borrow_mut().remove(&instance_id));
}

/// Release every PHP value held by the keyed context map.
///
/// This is the one piece of context state that *must* be freed on every request, including one
/// where the extension is disabled. Unlike `CONTEXT_STORAGE`, which holds plain Rust
/// `Arc<Context>`s, these are `ZVal`s: PHP memory, freed with the request's object store. A
/// thread-local holding them past RSHUTDOWN on a reused worker is a use-after-free on the next
/// request, not a leak. The classes stay registered when the extension is disabled, so
/// `Context::with()` is reachable either way and can populate this map either way.
pub fn clear_context_values() {
    CONTEXT_VALUES.with(|values| values.borrow_mut().clear());
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

/// The outcome of a detach, in the terms the spec defines.
///
/// `detach()` returns zero for an expected call and a non-zero bitmask otherwise, so these
/// map onto the `ScopeInterface` constants rather than onto a plain success/failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetachResult {
    /// The scope was the active one and was detached.
    Detached,
    /// The scope is not on the guard stack at all — most often a second `detach()` on a
    /// scope that was already detached.
    AlreadyDetached,
    /// The scope is on the stack but is not the active one, i.e. an out-of-order detach.
    Mismatch,
}

impl DetachResult {
    /// Whether the scope was actually detached, for callers continuing their teardown.
    pub fn is_detached(self) -> bool {
        self == DetachResult::Detached
    }

    /// The value `ScopeInterface::detach()` returns: zero, or a non-zero bitmask.
    ///
    /// The constants are `1 << (PHP_INT_SIZE << 3) - N`, i.e. relative to the top of PHP's
    /// integer, so they are computed from `i64::BITS` rather than written out. Hardcoding
    /// the 64-bit values would silently produce the wrong flags on a 32-bit build, where
    /// PHP's own constants are different numbers.
    ///
    /// Two parts of upstream's bitmask are deliberately not reproduced:
    ///
    /// - `INACTIVE`, which upstream sets when the scope belongs to a *different execution
    ///   context* than the current one — a fiber case. This extension does not track
    ///   execution contexts at all, so there is no state from which to decide it, and
    ///   claiming it for an out-of-order detach would mean something different from what
    ///   upstream means by it.
    /// - the *depth* upstream ORs into `MISMATCH`, saying how far down the stack the scope
    ///   was. Upstream detaches it anyway, splicing it out of the middle; we refuse. A depth
    ///   for a detach that did not happen would be misleading.
    pub fn to_php_flags(self) -> i64 {
        const DETACHED: i64 = 1i64 << (i64::BITS - 1);
        const MISMATCH: i64 = 1i64 << (i64::BITS - 3);

        match self {
            DetachResult::Detached => 0,
            DetachResult::AlreadyDetached => DETACHED,
            DetachResult::Mismatch => MISMATCH,
        }
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
/// Returns which of the spec's conditions applied, so callers can both skip the rest of
/// their teardown when the request was refused and report the reason to userland.
pub fn detach_context(instance_id: Option<u64>) -> DetachResult {
    if let Some(id) = instance_id {
        debug!("Detaching context instance {}", id);
        let outcome = GUARD_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            // Inspect the top before popping: popping first and checking afterwards would
            // already have dropped the guard, deactivating a scope nobody asked to detach.
            match stack.last() {
                Some((_, stack_id)) if *stack_id == id => {
                    stack.pop();
                    DetachResult::Detached
                }
                Some((_, stack_id)) => {
                    debug!(
                        "Not detaching context instance {}, is not top-most (top is {})",
                        id, stack_id
                    );
                    // Distinguish "this scope is somewhere further down the stack" from
                    // "this scope is not on the stack at all": the first is an out-of-order
                    // detach, the second a scope that was already detached. Userland gets
                    // MISMATCH for one and DETACHED for the other, which is the whole point
                    // of the return value.
                    if stack.iter().any(|(_, stack_id)| *stack_id == id) {
                        DetachResult::Mismatch
                    } else {
                        DetachResult::AlreadyDetached
                    }
                }
                None => {
                    debug!("Not detaching context instance {}, guard stack is empty", id);
                    DetachResult::AlreadyDetached
                }
            }
        });
        // Outside the GUARD_STACK borrow: this touches CONTEXT_STORAGE, and the dropped
        // guard above may run arbitrary Drop code.
        if outcome == DetachResult::Detached && !detached_context_holds_span(id) {
            maybe_remove_context_instance(Some(id));
        }
        outcome
    } else {
        // No id means nothing was ever stored for this scope, so there is nothing active to
        // detach. Upstream reports that as DETACHED rather than as success.
        DetachResult::AlreadyDetached
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

/// Get the `Arc<Context>` a PHP Context object is holding, falling back to the current one.
///
/// The argument is typed `ContextInterface`, so the object may be a userland implementation
/// rather than our `Context` class. `as_state_obj` is only sound for a class this extension
/// defined -- on anything else it reinterprets unrelated memory as our state -- so the class
/// is checked first, and a foreign object is treated as "no context of its own", which
/// resolves to the current context.
fn resolve_context_from_object(object: &mut ZObj) -> Arc<Context> {
    let is_ours = ClassEntry::from_globals(CONTEXT_CLASS_NAME)
        .is_ok_and(|expected| object.get_class().is_instance_of(expected));
    if is_ours {
        let state_obj = unsafe { object.as_state_obj::<Option<Arc<Context>>>() };
        if let Some(context) = state_obj.as_state().as_ref() {
            return context.clone();
        }
    }
    current_context()
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
        // An instance method, not static: that is how upstream declares it on
        // `ContextStorageInterface`, and a static declaration cannot satisfy an interface's
        // instance method.
        .add_method("current", Visibility::Public, move |_, _| {
            // `current_context` consults the guard stack, so an attached context is returned
            // rather than a fresh one, and the object carries that context's `context_id` so
            // its PHP-side values stay reachable. This is the same answer
            // `Context::getCurrent()` gives, which callers are entitled to expect.
            let context = current_context();
            let mut object = context_ce.clone().init_object()?;
            *object.as_mut_state() = Some(context);
            object.set_property(
                "context_id",
                current_context_instance_id().unwrap_or(0) as i64,
            );
            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME))));

    let scope_ce_attach = scope_ce.clone();
    class
        .add_method("attach", Visibility::Public, move |_, arguments| {
            let context_obj: &mut ZObj = arguments[0].expect_mut_z_obj()?;
            // `get_instance_id` rather than the `context_id` property directly: it maps 0 to
            // None, where reading the property gives `Some(0)` -- an id that resolves to no
            // stored context.
            let opt_instance_id = get_instance_id(context_obj);

            // A context with no stored instance is the ordinary case, not an error:
            // `Context::getCurrent()` reports 0 whenever nothing is attached yet, so
            // `$storage->attach(Context::getCurrent())` -- the first thing any caller of the
            // storage API does -- arrives here with None. Storing the context the object is
            // holding is what gives it an id to attach.
            let instance_id = match opt_instance_id {
                Some(id) => Some(id),
                None => {
                    let context = resolve_context_from_object(context_obj);
                    let id = store_context_instance(context);
                    // Carry the PHP-side values across, since attaching mints a new id for
                    // the same context -- the same reason `Context::activate()` inherits.
                    inherit_context_values(opt_instance_id, id);
                    id
                }
            };
            attach_context(instance_id).map_err(phper::Error::boxed)?;

            let mut object = scope_ce_attach.init_object()?;
            if let Some(id) = instance_id {
                object.set_property("context_id", id as i64);
            }
            Ok::<_, phper::Error>(object)
        })
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME))))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_STORAGE_SCOPE_INTERFACE_NAME,
        ))));

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
        .return_type(
            ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
                CONTEXT_STORAGE_SCOPE_INTERFACE_NAME,
            )))
            .allow_null(),
        );
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
    // Holds `ZVal`s, so this must run while the request's object store is still alive. This
    // call site does not cover every request -- RSHUTDOWN returns early when the extension is
    // disabled, yet the PHP classes stay registered and can still populate the map -- so
    // `clear_context_values` is also called unconditionally, ahead of that early return.
    // Clearing twice is harmless; relying on this call alone is a use-after-free next request.
    clear_context_values();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_successful_detach_is_the_only_zero() {
        // The spec's contract is "zero indicating an expected call, or a non-zero value
        // indicating that the call was unexpected", so zero has to mean exactly one thing.
        assert_eq!(DetachResult::Detached.to_php_flags(), 0);
        assert_ne!(DetachResult::AlreadyDetached.to_php_flags(), 0);
        assert_ne!(DetachResult::Mismatch.to_php_flags(), 0);
    }

    #[test]
    fn the_flags_match_upstreams_constants_on_a_64_bit_build() {
        // Upstream computes these as `1 << (PHP_INT_SIZE << 3) - N`. Pinning the arithmetic
        // against the literal values catches an off-by-one in the shift, which would
        // otherwise only show up as userland comparing against the wrong constant.
        assert_eq!(
            DetachResult::AlreadyDetached.to_php_flags(),
            -9223372036854775808i64,
            "DETACHED"
        );
        assert_eq!(
            DetachResult::Mismatch.to_php_flags(),
            2305843009213693952i64,
            "MISMATCH"
        );
    }

    #[test]
    fn the_flags_are_distinguishable_by_bitmask() {
        // Userland tests these with `$flags & ScopeInterface::DETACHED`, so each flag must
        // be isolated: one condition's value must not have another's bit set.
        const DETACHED: i64 = 1i64 << (i64::BITS - 1);
        const MISMATCH: i64 = 1i64 << (i64::BITS - 3);

        let already = DetachResult::AlreadyDetached.to_php_flags();
        let mismatch = DetachResult::Mismatch.to_php_flags();
        assert_ne!(already & DETACHED, 0);
        assert_eq!(already & MISMATCH, 0);
        assert_ne!(mismatch & MISMATCH, 0);
        assert_eq!(mismatch & DETACHED, 0);
    }

    #[test]
    fn only_a_real_detach_continues_teardown() {
        // `is_detached` gates removing the local root span; a refused detach that reported
        // true would drop a context userland can still reach.
        assert!(DetachResult::Detached.is_detached());
        assert!(!DetachResult::AlreadyDetached.is_detached());
        assert!(!DetachResult::Mismatch.is_detached());
    }
}