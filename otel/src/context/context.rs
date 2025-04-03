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

const CONTEXT_CLASS_NAME: &str = r"OpenTelemetry\Context\Context";
pub type ContextClass = StateClass<Option<Context>>;

// When a Context is activated, it is stored in CONTEXT_STORAGE, and a reference to the
// context created and stored as a class property.
thread_local! {
    static CONTEXT_STORAGE: RefCell<HashMap<u64, Context>> = RefCell::new(HashMap::new());
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

#[derive(Debug)]
struct Test(String);

pub fn make_context_class(
    scope_class: ScopeClass,
) -> ClassEntity<Option<Context>> {
    let mut class =
        ClassEntity::<Option<Context>>::new_with_default_state_constructor(CONTEXT_CLASS_NAME);
    let context_class = class.bind_class();

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class.add_static_method("getCurrent", Visibility::Public, move |_| {
        let context = Context::current();
        let mut object = context_class.clone().init_object()?;
        *object.as_mut_state() = Some(context);
        Ok::<_, phper::Error>(object)
    });

    //TODO: this doesn't usefully work, since the "key" is the type of the struct,
    // which needs to be created ahead of time. And it can only store scalar values.
    // see https://github.com/open-telemetry/opentelemetry-rust/blob/opentelemetry-0.28.0/opentelemetry/src/context.rs#L391
    class.add_method("with", Visibility::Public, |this, arguments| {
        let context: &mut Context = this.as_mut_state().as_mut().unwrap();
        let mut new = take(context);
        let php_value = arguments[1].expect_z_str()?.to_str()?.to_string();
        new = new.with_value(Test(php_value));
        *context = new;
        Ok::<_, phper::Error>(this.to_ref_owned())
    })
        .argument(Argument::by_val("key"))
        .argument(Argument::by_val("value"));

    class.add_method("get", Visibility::Public, |this, _arguments| -> phper::Result<ZVal> {
        let context: &mut Context = this.as_mut_state().as_mut().unwrap();
        let value = context.get::<Test>();
        match value {
            Some(value) => Ok::<_, phper::Error>(ZVal::from(value.0.as_str())),
            None => Ok(ZVal::default()),
        }
    })
        .argument(Argument::by_val("key"));

    class.add_method("activate", Visibility::Public, move |this, _arguments| {
        let ctx = this.as_mut_state().take().expect("No context stored!");

        let instance_id = new_instance_id();
        CONTEXT_STORAGE.with(|storage| storage.borrow_mut().insert(instance_id, ctx.clone()));
        this.set_property("context_id", instance_id as i64);

        let guard = ctx.attach();

        let mut object = scope_class.init_object()?;
        *object.as_mut_state() = Some(guard);
        Ok::<_, phper::Error>(object)
    });

    class
}
