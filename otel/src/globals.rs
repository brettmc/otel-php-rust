use crate::{
    context::propagation::text_map_propagator_interface::TEXT_MAP_PROPAGATOR_INTERFACE_NAME,
    logs::{
        logger_provider::LoggerProviderClass,
        logger_provider_interface::LOGGER_PROVIDER_INTERFACE_NAME,
    },
    trace::{
        propagation::trace_context_propagator::TraceContextPropagatorClass,
        tracer_provider::TracerProviderClass,
        tracer_provider_interface::TRACER_PROVIDER_INTERFACE_NAME,
    },
};
use phper::{
    classes::{ClassEntity, Visibility},
    functions::ReturnType,
    types::ReturnTypeHint,
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
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            TRACER_PROVIDER_INTERFACE_NAME,
        ))));

    class
        .add_static_method("propagator", Visibility::Public, move |_| {
            let object = propagator_class.init_object()?;
            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            TEXT_MAP_PROPAGATOR_INTERFACE_NAME,
        ))));

    class
        .add_static_method("loggerProvider", Visibility::Public, move |_| {
            let object = logger_provider_class.init_object()?;
            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            LOGGER_PROVIDER_INTERFACE_NAME,
        ))));

    class
}
