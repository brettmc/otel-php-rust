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
use opentelemetry::ContextGuard;

const SCOPE_CLASS_NAME: &str = r"OpenTelemetry\API\Context\Scope";
pub type ScopeClass = StateClass<Option<ContextGuard>>;

pub fn make_scope_class() -> ClassEntity<Option<ContextGuard>> {
    let mut class =
        ClassEntity::<Option<ContextGuard>>::new_with_default_state_constructor(SCOPE_CLASS_NAME);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("detach", Visibility::Public, |this, _| -> phper::Result<()> {
            let _ = this.as_mut_state().take();
            Ok(())
        });

    class
}
