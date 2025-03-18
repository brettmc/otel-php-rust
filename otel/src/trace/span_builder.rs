use phper::{
    classes::{ClassEntity, StateClass, Visibility},
    alloc::ToRefOwned,
};
use std::{
    convert::Infallible,
};
use opentelemetry::{
    Context,
    KeyValue,
    trace::{
        SpanBuilder,
        Tracer,
    }
};
use opentelemetry_sdk::trace::SdkTracer;
use crate::trace::{
    span::SpanClass,
};

pub struct MySpanBuilder {
    span_builder: Option<SpanBuilder>,
    tracer: Option<SdkTracer>,
}
impl MySpanBuilder {
    pub fn new(span_builder: SpanBuilder, tracer: SdkTracer) -> Self {
        Self { span_builder: Some(span_builder), tracer: Some(tracer)}
    }
    pub fn empty() -> Self {
        Self{ span_builder: None, tracer: None}
    }
}

const SPAN_BUILDER_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\SpanBuilder";

pub type SpanBuilderClass = StateClass<MySpanBuilder>;

pub fn make_span_builder_class(span_class: SpanClass) -> ClassEntity<MySpanBuilder> {
    let mut class = ClassEntity::<MySpanBuilder>::new_with_state_constructor(
        SPAN_BUILDER_CLASS_NAME,
        || {
            MySpanBuilder::empty()
        },
    );

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    //TODO setParent, addLink, setAttributes, setStartTimestamp, setSpanKind

    class.add_method("setAttribute", Visibility::Public, |this, arguments| {
        let state = this.as_mut_state();
        let span_builder = state.span_builder.as_ref().expect("SpanBuilder not set");
        let name = arguments[0].expect_z_str()?.to_str()?.to_string();
        let value = arguments[1].expect_z_str()?.to_str()?.to_string();
        let mut attributes = span_builder.attributes.clone().unwrap_or_default();
        attributes.push(KeyValue::new(name, value));
        let new_span_builder = span_builder.clone().with_attributes(attributes);
        state.span_builder = Some(new_span_builder);

        Ok::<_, phper::Error>(this.to_ref_owned())
    });

    class
        .add_method("startSpan", Visibility::Public, move |this, _| {
            let state = this.as_state();
            let span_builder = state.span_builder.as_ref().expect("SpanBuilder not set");
            let tracer = state.tracer.as_ref().expect("Tracer not set");

            let span = tracer.build_with_context(span_builder.clone(), &Context::current());
            tracing::debug!("SpanBuilder::Starting span");
            let mut object = span_class.init_object()?;
            *object.as_mut_state() = Some(span);
            Ok::<_, phper::Error>(object)
        });

    class
}
