use phper::{
    classes::{
        ClassEntity,
        Visibility,
    },
    functions::{ReturnType},
    types::{ReturnTypeHint},
};
use crate::trace::tracer_provider::TracerProviderClass;
const GLOBALS_CLASS_NAME: &str = r"OpenTelemetry\API\Globals";

pub fn make_globals_class(tracer_provider_class: TracerProviderClass) -> ClassEntity<()> {
    let mut class = ClassEntity::new(GLOBALS_CLASS_NAME);

    class
        .add_static_method("tracerProvider", Visibility::Public, move |_| {
            let object = tracer_provider_class.init_object()?;
            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\API\Trace\TracerProviderInterface"))));

    /*class
        .add_static_method("propagator", Visibility::Public, move |_| {
            todo!("OpenTelemetry propagator");
        });*/

    class
}