use crate::context::{
    context::ContextClassEntity,
    storage::{
        detach_context,
        get_context_instance,
    }
};
use phper::{
    classes::{
        ClassEntity,
        StateClass,
        Visibility,
    },
};
use std::{
    convert::Infallible,
};
use opentelemetry::{
    ContextGuard,
};

const SCOPE_CLASS_NAME: &str = r"OpenTelemetry\Context\Scope";
pub type ScopeClass = StateClass<Option<ContextGuard>>;
pub type ScopeClassEntity = ClassEntity<Option<ContextGuard>>;

//TODO no longer directly wraps ContextGuard
pub fn new_scope_class() -> ScopeClassEntity {
    ClassEntity::<Option<ContextGuard>>::new_with_default_state_constructor(SCOPE_CLASS_NAME)
}

pub fn build_scope_class(
    class: &mut ScopeClassEntity,
    context_class: &ContextClassEntity,
) {
    let _scope_class = class.bound_class();
    let context_ce = context_class.bound_class();
    class.add_property("context_id", Visibility::Private, 0i64);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("detach", Visibility::Public, |_, _| -> phper::Result<()> {
            detach_context();
            Ok(())
        });

    class
        .add_method("context", Visibility::Public, move |this,_| {
            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            let ctx = get_context_instance(instance_id as u64);
            let mut object = context_ce.init_object()?;
            *object.as_mut_state() = ctx;
            object.set_property("context_id", instance_id as i64);
            Ok::<_, phper::Error>(object)
        });
}
