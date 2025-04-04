use crate::context::scope::ScopeClass;
use phper::{
    alloc::ToRefOwned,
    classes::{ClassEntity, StateClass, Visibility},
    functions::Argument,
    values::ZVal,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    convert::Infallible,
    mem::take,
    sync::atomic::{AtomicU64, Ordering},
};
use opentelemetry::{
    Context,
};

const CONTEXT_CLASS_NAME: &str = r"OpenTelemetry\Context\Storage";
pub type ContextClass = StateClass<()>;

// When a Context is activated, it is stored in CONTEXT_STORAGE, and a reference to the
// context created and stored as a class property.
thread_local! {
    static CONTEXT_STORAGE: RefCell<HashMap<u64, Context>> = RefCell::new(HashMap::new());
    // todo active scope stack?
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

fn new_instance_id() -> u64 {
    INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub fn make_context_class(
    scope_class: ScopeClass,
) -> ClassEntity<Option<Context>> {
    let mut class =
        ClassEntity::<Option<Context>>::new_with_default_state_constructor(CONTEXT_CLASS_NAME);
    let context_class = class.bound_class();

    //class.add_property("context_id", Visibility::Private, 0i64);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class.add_static_method("current", Visibility::Public, move |_| {
        let context = Context::current();
        let mut object = context_class.clone().init_object()?;
        *object.as_mut_state() = Some(context);
        Ok::<_, phper::Error>(object)
    }); //return ContextInterface

    class.add_method("attach", Visibility::Public, move |this, _arguments| {
        let ctx = this.as_mut_state().take().expect("No context stored!");

        let instance_id = new_instance_id();
        CONTEXT_STORAGE.with(|storage| storage.borrow_mut().insert(instance_id, ctx.clone()));
        this.set_property("context_id", instance_id as i64);

        let guard = ctx.attach();

        let mut object = scope_class.init_object()?;
        *object.as_mut_state() = Some(guard);
        object.set_property("context_id", instance_id as i64);
        Ok::<_, phper::Error>(object)
    })
    .argument(Argument::new("context"))
    .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\Scope"))));

    class.add_method("scope", Visibility::Public, move |this, _arguments| {

    }); //return ScopeInterface

    class
}
