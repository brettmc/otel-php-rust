use crate::context::{
    context::ContextClassEntity,
    scope::ScopeClassEntity,
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
};
use tracing::debug;

const CONTEXT_STORAGE_CLASS_NAME: &str = r"OpenTelemetry\Context\Storage";
pub type StorageClass = StateClass<()>;
pub type StorageClassEntity = ClassEntity<()>;

// When a Context is activated, it is stored in CONTEXT_STORAGE, and a reference to the
// context created and stored as a class property.
thread_local! {
    static CONTEXT_STORAGE: RefCell<HashMap<u64, Arc<Context>>> = RefCell::new(HashMap::new());
    static GUARD_STACK: RefCell<Vec<(ContextGuard, u64)>> = RefCell::new(Vec::new());
    static CONTEXT_GUARD_MAP: RefCell<HashMap<usize, ContextGuard>> = RefCell::new(HashMap::new()); //for observer use
}
static INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn current_context() -> Arc<Context> {
    current_context_instance_id()
        .and_then(|id| get_context_instance(Some(id)))
        .unwrap_or_else(|| Arc::new(Context::current()))
}

pub fn resolve_context(instance_id: Option<u64>) -> Arc<Context> {
    match instance_id {
        Some(id) => get_context_instance(Some(id)).expect("context not found"),
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

pub fn detach_context(instance_id: Option<u64>) {
    if let Some(id) = instance_id {
        debug!("Detaching context instance {}", id);
        GUARD_STACK.with(|stack| {
            stack.borrow_mut().pop().map(|(_guard, stack_id)| {
                if stack_id == id {
                    maybe_remove_context_instance(Some(stack_id));
                } else {
                    debug!("Not detaching context instance {}, is not top-most", id);
                }
            });
        });
    }
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
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ContextInterface"))));

    let scope_ce_attach = scope_ce.clone();
    class
        .add_method("attach", Visibility::Public, move |_, arguments| {
            let context_obj: &mut ZObj = arguments[0].expect_mut_z_obj()?;
            let instance_id = context_obj.get_property("context_id").as_long();
            let opt_instance_id = instance_id.map(|id| id as u64);
            attach_context(opt_instance_id).map_err(phper::Error::boxed)?;

            let mut object = scope_ce_attach.init_object()?;
            if let Some(id) = opt_instance_id {
                object.set_property("context_id", id as i64);
            }
            Ok::<_, phper::Error>(object)
        })
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ContextInterface"))))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ScopeInterface"))));

    let scope_ce_scope = scope_ce.clone();
    class
        .add_method("scope", Visibility::Public, move |_, _arguments| {
            let popped = GUARD_STACK.with(|stack| {
                stack.borrow_mut().pop()
            });
            match popped {
                Some((_guard, context_id)) => {
                    let mut object = scope_ce_scope.init_object()?;
                    object.set_property("context_id", context_id as i64);
                    Ok::<_, phper::Error>(object.into())
                }
                None => {
                    Ok::<_, phper::Error>(ZVal::from(()))
                }
            }
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ScopeInterface"))).allow_null());
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