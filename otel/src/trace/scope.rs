use phper::{
    classes::{
        ClassEntity,
        StaticStateClass,
        Visibility,
    },
};
use std::{
    convert::Infallible,
};
use opentelemetry::Context;

const SCOPE_CLASS_NAME: &str = "OpenTelemetry\\API\\Context\\Scope";

pub static SCOPE_CLASS: StaticStateClass<Option<Context>> = StaticStateClass::null();

pub fn make_scope_class() -> ClassEntity<Option<Context>> {
    let mut class =
        ClassEntity::<Option<Context>>::new_with_default_state_constructor(SCOPE_CLASS_NAME);

    class.bind(&SCOPE_CLASS);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("detach", Visibility::Public, |_this, _| -> phper::Result<()> {

            Ok(())
        });

    class
}
