use phper::{
    classes::{ClassEntity, StaticStateClass, Visibility},
};
use std::{
    convert::Infallible,
};
use std::mem::take;
use opentelemetry::trace::{
    SpanBuilder,
};
use opentelemetry::global;
use opentelemetry::global::BoxedSpan;
use crate::trace::span::SPAN_CLASS;

const SPAN_BUILDER_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\SpanBuilder";

pub static SPAN_BUILDER_CLASS: StaticStateClass<Option<SpanBuilder>> = StaticStateClass::null();

pub fn make_span_builder_class() -> ClassEntity<Option<SpanBuilder>> {
    let mut class =
        ClassEntity::<Option<SpanBuilder>>::new_with_default_state_constructor(SPAN_BUILDER_CLASS_NAME);

    class.bind(&SPAN_BUILDER_CLASS);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("startSpan", Visibility::Public, |this, _| {
            let state = take(this.as_mut_state());
            //TODO keep a reference to the tracer in this instance and use that!!
            let tracer = global::tracer("change-me");
            let builder = state.as_ref().expect("SpanBuilder is not initialized");
            let span: BoxedSpan = builder.clone().start(&tracer);
            let mut object = SPAN_CLASS.init_object()?;
            *object.as_mut_state() = Some(span);
            Ok::<_, phper::Error>(object)
        });

    class
}
