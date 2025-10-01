use phper::{
    classes::{
        ClassEntity,
        Visibility,
    },
    functions::ReturnType,
    types::ReturnTypeHint,
};
use crate::{
    logs::logger_provider::LoggerProviderClass,
    trace::{
        propagation::trace_context_propagator::TraceContextPropagatorClass,
        tracer_provider::TracerProviderClass,
    }
};
const GLOBALS_CLASS_NAME: &str = r"OpenTelemetry\API\Globals";

pub fn make_globals_class(
    tracer_provider_class: TracerProviderClass,
    propagator_class: TraceContextPropagatorClass,
    logger_provider_class: LoggerProviderClass,
) -> ClassEntity<()> {
    let mut class = ClassEntity::new(GLOBALS_CLASS_NAME);

    class
        .add_static_method("tracerProvider", Visibility::Public, move |_| {
            let object = tracer_provider_class.init_object()?;
            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\API\Trace\TracerProviderInterface"))));

    class
        .add_static_method("propagator", Visibility::Public, move |_| {
            let object = propagator_class.init_object()?;
            Ok::<_, phper::Error>(object)
        }); //TODO return interface

    class
        .add_static_method("loggerProvider", Visibility::Public, move |_| {
            let object = logger_provider_class.init_object()?;
            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\API\Logs\LoggerProvider"))));

    class
}