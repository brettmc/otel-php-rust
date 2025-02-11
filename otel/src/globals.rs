use phper::{
    classes::{ClassEntity, Visibility},
};
use opentelemetry::global;
use crate::tracer::TRACER_CLASS;
use crate::tracer_provider::TRACER_PROVIDER_CLASS;

const GLOBALS_CLASS_NAME: &str = "OpenTelemetry\\Globals";

pub fn make_globals_class() -> ClassEntity<()> {
    let mut class = ClassEntity::new(GLOBALS_CLASS_NAME);

    class.add_static_method("getTracer", Visibility::Public, |_arguments| {
        let tracer = global::tracer("test");
        let mut object = TRACER_CLASS.init_object()?;
        *object.as_mut_state() = Some(tracer);
        Ok::<_, phper::Error>(object)
    });

    class.add_static_method("tracerProvider", Visibility::Public, |_| {
        let provider = global::tracer_provider();
        let mut object = TRACER_PROVIDER_CLASS.init_object()?;
        *object.as_mut_state() = Some(provider);
        Ok::<_, phper::Error>(object)
    });

    class
}
