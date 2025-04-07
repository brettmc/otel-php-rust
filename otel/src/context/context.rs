use crate::context::{
    scope::ScopeClassEntity,
    storage::{self, StorageClassEntity},
};
use phper::{
    alloc::ToRefOwned,
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::{Argument, ReturnType},
    types::ReturnTypeHint,
    values::ZVal,
};
use std::{
    convert::Infallible,
    sync::Arc,
};
use opentelemetry::{
    Context,
};
use tracing::debug;

const CONTEXT_CLASS_NAME: &str = r"OpenTelemetry\Context\Context";
pub type ContextClass = StateClass<Option<Arc<Context>>>;
pub type ContextClassEntity = ClassEntity<Option<Arc<Context>>>;

#[derive(Debug)]
struct Test(String);

pub fn new_context_class() -> ContextClassEntity {
    ClassEntity::<Option<Arc<Context>>>::new_with_default_state_constructor(CONTEXT_CLASS_NAME)
}

pub fn build_context_class(
    class: &mut ContextClassEntity,
    scope_class: &ScopeClassEntity,
    storage_class: &StorageClassEntity,
    context_interface: Interface,
) {
    let context_class = class.bound_class();
    let scope_ce = scope_class.bound_class();
    let storage_ce = storage_class.bound_class();

    class.implements(context_interface);
    class.add_property("context_id", Visibility::Private, 0i64);

    class
        .add_method("__construct", Visibility::Private, |_, _| {
            Ok::<_, Infallible>(())
        });

    //TODO: on context destruct, delete from storage (iff not stored by scope)
    class
        .add_method("__destruct", Visibility::Public, |this, _| {
            let context_id = this.get_property("context_id").as_long().expect("invalid context_id stored");
            debug!("Context::__destruct for context_id = {}", context_id);
            if context_id > 0 {
                storage::maybe_remove_context_instance(context_id as u64);
            }
            Ok::<_, Infallible>(())
        });

    class
        .add_static_method("getCurrent", Visibility::Public, {
            let context_ce = context_class.clone();
            move |_| {
                let context = storage::current_context();

                let mut object = context_ce.init_object()?;
                *object.as_mut_state() = Some(context);
                Ok::<_, phper::Error>(object)
            }
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ContextInterface"))));


    //TODO: this doesn't usefully work, since the "key" is the type of the struct,
    // which needs to be created ahead of time. And it can only store scalar values.
    // see https://github.com/open-telemetry/opentelemetry-rust/blob/opentelemetry-0.28.0/opentelemetry/src/context.rs#L391
    class
        .add_method("with", Visibility::Public, |this, arguments| {
            let arc = this.as_mut_state().as_mut().unwrap();
            let mut new_ctx = (**arc).clone();
            let php_value = arguments[1].expect_z_str()?.to_str()?.to_string();
            new_ctx = new_ctx.with_value(Test(php_value));
            *arc = Arc::new(new_ctx);
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("key"))
        .argument(Argument::new("value"));

    class
        .add_method("get", Visibility::Public, |this, _arguments| -> phper::Result<ZVal> {
            let arc = this.as_state().as_ref().unwrap();
            let context = &**arc; // deref Arc<Context>
            let value = context.get::<Test>();
            match value {
                Some(value) => Ok::<_, phper::Error>(ZVal::from(value.0.as_str())),
                None => Ok(ZVal::default()),
            }
        })
        .argument(Argument::new("key"));

    class
        .add_method("activate", Visibility::Public, {
            let scope_ce = scope_ce.clone();
            move |this, _arguments| {
                let arc = this.as_state().as_ref().unwrap().clone(); // Arc<Context>
                let instance_id = storage::store_context_instance(arc.clone());
                debug!("Storing context: {}", instance_id);

                this.set_property("context_id", instance_id as i64);
                storage::attach_context(instance_id).map_err(phper::Error::boxed)?;

                let mut object = scope_ce.init_object()?;
                *object.as_mut_state() = None;
                object.set_property("context_id", instance_id as i64);

                Ok::<_, phper::Error>(object)
            }
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ScopeInterface"))));

    class
        .add_static_method("storage", Visibility::Public, move |_| {
            let object = storage_ce.init_object()?;
            Ok::<_, phper::Error>(object)
        });

}
