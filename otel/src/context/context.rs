use crate::context::{
    scope::ScopeClassEntity,
    storage::{
        attach_context,
        current_context_instance_id,
        get_context_instance,
        store_context_instance,
        StorageClassEntity,
    },
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
    mem::take,
};
use opentelemetry::{
    Context,
};

const CONTEXT_CLASS_NAME: &str = r"OpenTelemetry\Context\Context";
pub type ContextClass = StateClass<Option<Context>>;
pub type ContextClassEntity = ClassEntity<Option<Context>>;

#[derive(Debug)]
struct Test(String);

pub fn new_context_class() -> ContextClassEntity {
    ClassEntity::<Option<Context>>::new_with_default_state_constructor(CONTEXT_CLASS_NAME)
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

    //TODO: on context destruct, delete from storage (iff not stored by scope)
    class
        .add_method("__construct", Visibility::Private, |_, _| {
            Ok::<_, Infallible>(())
        });

    class
        .add_static_method("getCurrent", Visibility::Public, {
            let context_ce = context_class.clone();
            move |_| {
                let context = match current_context_instance_id()
                    .and_then(|id| get_context_instance(id))
                {
                    Some(ctx) => ctx,
                    None => Context::current(), // fallback to OpenTelemetry's global context
                };

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
            let context: &mut Context = this.as_mut_state().as_mut().unwrap();
            let mut new = take(context);
            let php_value = arguments[1].expect_z_str()?.to_str()?.to_string();
            new = new.with_value(Test(php_value));
            *context = new;
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("key"))
        .argument(Argument::new("value"));

    class
        .add_method("get", Visibility::Public, |this, _arguments| -> phper::Result<ZVal> {
            let context: &mut Context = this.as_mut_state().as_mut().unwrap();
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
                // Clone the context and drop the mutable borrow ASAP
                let instance_id = {
                    let context: &mut Context = this.as_mut_state().as_mut().unwrap();
                    store_context_instance(context.clone())
                };

                // Store the ID on the PHP object
                this.set_property("context_id", instance_id as i64);

                // Attach the context via storage (push guard to stack)
                attach_context(instance_id).map_err(phper::Error::boxed)?;

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
