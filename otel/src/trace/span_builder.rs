use phper::{
    classes::{ClassEntity, StateClass, Visibility},
    alloc::ToRefOwned,
};
use std::{
    convert::Infallible,
    mem::take,
};
use opentelemetry::{
    Context,
    InstrumentationScope,
    KeyValue,
    trace::{
        SpanBuilder,
        Tracer,
        TracerProvider,
    }
};
use crate::trace::{
    span::SpanClass,
    tracer_provider::get_tracer_provider,
};

const SPAN_BUILDER_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\SpanBuilder";

pub type SpanBuilderClass = StateClass<Option<SpanBuilder>>;

pub fn make_span_builder_class(span_class: SpanClass) -> ClassEntity<Option<SpanBuilder>> {
    let mut class =
        ClassEntity::<Option<SpanBuilder>>::new_with_default_state_constructor(SPAN_BUILDER_CLASS_NAME);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    //TODO setParent, addLink, setAttributes, setStartTimestamp, setSpanKind

    class.add_method("setAttribute", Visibility::Public, |this, arguments| {
        let span_builder: &mut SpanBuilder = this.as_mut_state().as_mut().unwrap();
        let mut builder = take(span_builder);
        let name = arguments[0].expect_z_str()?.to_str()?.to_string();
        let value = arguments[1].expect_z_str()?.to_str()?.to_string();
        let attributes = [KeyValue::new(name, value)];
        builder = builder.with_attributes(attributes);
        *span_builder = builder;
        Ok::<_, phper::Error>(this.to_ref_owned())
    });

    class
        .add_method("startSpan", Visibility::Public, move |this, _| {
            let span_builder = take(this.as_mut_state()).expect("SpanBuilder missing");
            let provider = get_tracer_provider();
            let scope = InstrumentationScope::builder("php_rust")
                .build();
            let tracer = provider.tracer_with_scope(scope);
            let span = tracer.build_with_context(span_builder, &Context::current());
            tracing::debug!("SpanBuilder::Starting span");
            let mut object = span_class.init_object()?;
            *object.as_mut_state() = Some(span);
            Ok::<_, phper::Error>(object)
        });

    class
}
