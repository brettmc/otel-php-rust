use phper::{
    classes::{ClassEntity, StateClass, Visibility},
    functions::Argument,
};
use std::{
    convert::Infallible,
};
use opentelemetry::{
    trace::{
        SpanBuilder,
        Tracer,
    }
};
use opentelemetry_sdk::trace::SdkTracer;
use crate::trace::span_builder::SpanBuilderClass;

pub type TracerClass = StateClass<Option<SdkTracer>>;

const TRACER_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\Tracer";

pub fn make_tracer_class(span_builder_class: SpanBuilderClass) -> ClassEntity<Option<SdkTracer>> {
    let mut class =
        ClassEntity::<Option<SdkTracer>>::new_with_default_state_constructor(TRACER_CLASS_NAME);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("spanBuilder", Visibility::Public, move |this, arguments| {
            let tracer: &SdkTracer = this.as_state().as_ref().unwrap();
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();
            let span_builder: SpanBuilder = tracer.span_builder(name);
            let mut object = span_builder_class.init_object()?;
            *object.as_mut_state() = Some(span_builder);
            Ok::<_, phper::Error>(object)
        })
        .argument(Argument::by_val("name"));

    class
}
