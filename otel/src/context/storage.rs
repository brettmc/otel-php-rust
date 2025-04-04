use crate::context::{
    context::ContextClassEntity,
    scope::ScopeClassEntity,
};
use phper::{
    classes::{ClassEntity, StateClass, Visibility},
    functions::{Argument, ReturnType},
    objects::ZObj,
    types::{ArgumentTypeHint, ReturnTypeHint},
    values::ZVal,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    convert::Infallible,
    sync::atomic::{AtomicU64, Ordering},
};
use opentelemetry::{
    Context,
    ContextGuard,
};

const CONTEXT_STORAGE_CLASS_NAME: &str = r"OpenTelemetry\Context\Storage";
pub type StorageClass = StateClass<()>;
pub type StorageClassEntity = ClassEntity<()>;

// When a Context is activated, it is stored in CONTEXT_STORAGE, and a reference to the
// context created and stored as a class property.
thread_local! {
    static CONTEXT_STORAGE: RefCell<HashMap<u64, Context>> = RefCell::new(HashMap::new());
    static GUARD_STACK: RefCell<Vec<(ContextGuard, u64)>> = RefCell::new(Vec::new());

}
static INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn get_context_instance(instance_id: u64) -> Option<Context> {
    if instance_id == 0 {
        None
    } else {
        CONTEXT_STORAGE.with(|storage| storage.borrow().get(&instance_id).cloned())
    }
}

pub fn store_context_instance(context: Context) -> u64 {
    let instance_id = new_instance_id();
    CONTEXT_STORAGE.with(|storage| {
        storage.borrow_mut().insert(instance_id, context)
    });

    instance_id
}

pub fn attach_context(instance_id: u64) -> Result<(), &'static str> {
    let context = get_context_instance(instance_id).ok_or("Context not found")?;
    let guard = context.attach();
    GUARD_STACK.with(|stack| {
        stack.borrow_mut().push((guard, instance_id));
    });
    Ok(())
}

pub fn detach_context() -> Option<u64> {
    GUARD_STACK.with(|stack| {
        stack.borrow_mut().pop().map(|(_guard, id)| id)
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
) {
    let _storage_ce = class.bound_class();
    let scope_ce = scope_class_entity.bound_class();
    let context_ce = context_class_entity.bound_class();

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_static_method("current", Visibility::Public, move |_| {
            //todo store our own stack of (guard, context) ?
            let context = Context::current();
            let mut object = context_ce.clone().init_object()?;
            *object.as_mut_state() = Some(context);
            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\Context"))));

    let scope_ce_attach = scope_ce.clone();
    class
        .add_method("attach", Visibility::Public, move |_, arguments| {
            let context_obj: &mut ZObj = arguments[0].expect_mut_z_obj()?;
            let instance_id = context_obj.get_property("context_id").as_long().unwrap_or(0);
            //let context = get_context_instance(instance_id as u64).expect("context not found");
            attach_context(instance_id as u64).map_err(phper::Error::boxed)?;

            let mut object = scope_ce_attach.init_object()?;
            *object.as_mut_state() = None;
            object.set_property("context_id", instance_id as i64);
            Ok::<_, phper::Error>(object)
        })
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\Context"))))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\Scope"))));

    let scope_ce_scope = scope_ce.clone();
    class
        .add_method("scope", Visibility::Public, move |_, _arguments| {
            let popped = GUARD_STACK.with(|stack| {
                stack.borrow_mut().pop()
            });
            match popped {
                Some((guard, context_id)) => {
                    let mut object = scope_ce_scope.init_object()?;
                    *object.as_mut_state() = Some(guard);
                    object.set_property("context_id", context_id as i64);
                    Ok::<_, phper::Error>(object.into())
                }
                None => {
                    Ok::<_, phper::Error>(ZVal::from(()))
                }
            }
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\Scope"))).allow_null());
}
