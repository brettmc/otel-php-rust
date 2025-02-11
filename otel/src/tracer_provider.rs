use phper::{
    classes::{ClassEntity, StaticStateClass, Visibility},
};
use std::{
    convert::Infallible,
};
use std::mem::take;
use opentelemetry::global::GlobalTracerProvider;
use opentelemetry::trace::TracerProvider;
use crate::tracer::TRACER_CLASS;

const TRACER_PROVIDER_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\TracerProvider";

pub static TRACER_PROVIDER_CLASS: StaticStateClass<Option<GlobalTracerProvider>> = StaticStateClass::null();

pub fn make_tracer_provider_class() -> ClassEntity<Option<GlobalTracerProvider>> {
    let mut class =
        ClassEntity::<Option<GlobalTracerProvider>>::new_with_default_state_constructor(TRACER_PROVIDER_CLASS_NAME);

    class.bind(&TRACER_PROVIDER_CLASS);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class.add_method("getTracer", Visibility::Public, |this, arguments| {
        let state = take(this.as_mut_state());
        let name = arguments[0].expect_z_str()?.to_str()?.to_string();
        let tracer = state.as_ref().expect("TracerProvider is not initialized").tracer(name);
        let mut object = TRACER_CLASS.init_object()?;
        *object.as_mut_state() = Some(tracer);
        Ok::<_, phper::Error>(object)
    });

    class
}
