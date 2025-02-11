use phper::{
    // alloc::ToRefOwned,
    classes::{ClassEntity, StaticStateClass, Visibility},
    functions::Argument,
};
use std::{
    convert::Infallible,
    //mem::take,
    // time::Duration
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
use opentelemetry::global;

const GLOBALS_CLASS_NAME: &str = "OpenTelemetry\\Globals";

const TRACER_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\Tracer";

static TRACER_CLASS: StaticStateClass<Option<BoxedTracer>> = StaticStateClass::null();

pub fn make_globals_class() -> ClassEntity<()> {
    // `new_with_default_state_constructor` means initialize the state of
    // `ClientBuilder` as `Default::default`.
    let mut class = ClassEntity::new(GLOBALS_CLASS_NAME);

    // Inner call the `ClientBuilder::build`, and wrap the result `Client` in
    // Object.
    class.add_method("getTracer", Visibility::Public, |_this, _arguments| {
        //let state = take(this.as_mut_state());
        //let client = ClientBuilder::build(state).map_err(HttpClientError::Reqwest)?;
        let tracer = global::tracer("test");
        let mut object = TRACER_CLASS.init_object()?;
        *object.as_mut_state() = Some(tracer);
        Ok::<_, phper::Error>(object)
    });

    class
}

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
