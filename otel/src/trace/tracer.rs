use phper::{
    classes::{ClassEntity, StaticStateClass, Visibility},
    functions::Argument,
};
use std::{
    convert::Infallible,
};
use opentelemetry::trace::{
    SpanKind,
    Span,
    Status,
    Tracer,
};
use opentelemetry::global::{
    BoxedTracer,
};

const TRACER_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\Tracer";

pub static TRACER_CLASS: StaticStateClass<Option<BoxedTracer>> = StaticStateClass::null();

pub fn make_tracer_class() -> ClassEntity<Option<BoxedTracer>> {
    let mut class =
        ClassEntity::<Option<BoxedTracer>>::new_with_default_state_constructor(TRACER_CLASS_NAME);

    class.bind(&TRACER_CLASS);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("test", Visibility::Public, |this, arguments| -> phper::Result<()> {
            let tracer = this.as_state().as_ref().unwrap();
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();
            let mut span = tracer
                .span_builder(name)
                .with_kind(SpanKind::Server)
                .start(tracer);
            span.set_status(Status::Ok);
            span.end();
            Ok(())
        })
        .argument(Argument::by_val("name"));

    class
}
