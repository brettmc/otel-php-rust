use phper::{
    classes::{ClassEntity, StateClass, Visibility},
    alloc::ToRefOwned,
};
use std::{
    convert::Infallible,
    mem::take,
};
use opentelemetry::{
    KeyValue,
    global::{
        self,
        BoxedSpan,
    },
    trace::{
        SpanBuilder,
    }
};
use crate::trace::span::SpanClass;

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
            let state = take(this.as_mut_state());
            //TODO: store+use tracer used to build this
            let tracer = global::tracer("default");
            let builder = state.as_ref().expect("SpanBuilder is not initialized");
            let span: BoxedSpan = builder.clone().start(&tracer);
            let mut object = span_class.init_object()?;
            *object.as_mut_state() = Some(span);
            Ok::<_, phper::Error>(object)
        });

    class
}
