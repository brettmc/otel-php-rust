use phper::{
    alloc::ToRefOwned,
    classes::{ClassEntity, StateClass, Visibility},
    functions::Argument,
    values::ZVal,
};
use std::{
    convert::Infallible,
    mem::take,
};
use opentelemetry::{
    Context,
};

const CONTEXT_CLASS_NAME: &str = "OpenTelemetry\\Context\\Context";
pub type ContextClass = StateClass<Option<Context>>;

#[derive(Debug)]
struct Test(String);

pub fn make_context_class() -> ClassEntity<Option<Context>> {
    let mut class =
        ClassEntity::<Option<Context>>::new_with_default_state_constructor(CONTEXT_CLASS_NAME);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class.add_static_method("getCurrent", Visibility::Public, |_| {
        let context = Context::current();
        let mut object = class.init_object()?;
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

    class
}
