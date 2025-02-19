use phper::{
    classes::{
        ClassEntity,
        Visibility,
    },
};
use opentelemetry::global;
use crate::trace::tracer_provider::TracerProviderClass;

const GLOBALS_CLASS_NAME: &str = "OpenTelemetry\\API\\Globals";

pub fn make_globals_class(tracer_provider_class: TracerProviderClass) -> ClassEntity<()> {
    let mut class = ClassEntity::new(GLOBALS_CLASS_NAME);

    class.add_static_method("tracerProvider", Visibility::Public, move |_| {
        let provider = global::tracer_provider();
        let mut object = tracer_provider_class.init_object()?;
        *object.as_mut_state() = Some(provider);
        Ok::<_, phper::Error>(object)
    });

    class
}
