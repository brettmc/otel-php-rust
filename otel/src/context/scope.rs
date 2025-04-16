use crate::{
    context::{
        context::ContextClassEntity,
        storage,
    },
    trace::local_root_span,
};
use phper::{
    classes::{
        ClassEntity,
        Interface,
        StateClass,
        Visibility,
    },
    functions::ReturnType,
    types::ReturnTypeHint,
};
use std::{
    convert::Infallible,
};

const SCOPE_CLASS_NAME: &str = r"OpenTelemetry\Context\Scope";
pub type ScopeClass = StateClass<()>;
pub type ScopeClassEntity = ClassEntity<()>;

pub fn new_scope_class() -> ScopeClassEntity {
    ScopeClassEntity::new_with_default_state_constructor(SCOPE_CLASS_NAME)
}

pub fn build_scope_class(
    class: &mut ScopeClassEntity,
    context_class: &ContextClassEntity,
    scope_interface: &Interface,
) {
    let _scope_class = class.bound_class();
    let context_ce = context_class.bound_class();
    class.implements(scope_interface.clone());
    class.add_property("context_id", Visibility::Private, 0i64);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("detach", Visibility::Public, |this, _| -> phper::Result<()> {
            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            if instance_id > 0 {
                storage::detach_context(instance_id as u64);
                local_root_span::maybe_remove_local_root_span(instance_id as u64);
            }
            Ok(())
        })
        .return_type(ReturnType::new(ReturnTypeHint::Int));

    class
        .add_method("context", Visibility::Public, move |this,_| {
            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            let ctx = storage::get_context_instance(instance_id as u64);
            let mut object = context_ce.init_object()?;
            *object.as_mut_state() = ctx;
            object.set_property("context_id", instance_id as i64);
            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\Context\ContextInterface"))));
}
