use phper::{
    classes::{ClassEntity, StaticStateClass, Visibility},
};
use std::{
    convert::Infallible,
};
use opentelemetry::trace::{
    SpanContext,
    TraceFlags,
    TraceState,
    SpanId,
    TraceId,
};

const SPAN_CONTEXT_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\SpanContext";

pub static SPAN_CONTEXT_CLASS: StaticStateClass<Option<SpanContext>> = StaticStateClass::null();

pub fn make_span_context_class() -> ClassEntity<Option<SpanContext>> {
    let mut class =
        ClassEntity::<Option<SpanContext>>::new_with_default_state_constructor(SPAN_CONTEXT_CLASS_NAME);

    class.bind(&SPAN_CONTEXT_CLASS);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class.add_static_method("getInvalid", Visibility::Public, |_| {
        let span_context = SpanContext::empty_context();
        let mut object = SPAN_CONTEXT_CLASS.init_object()?;
        *object.as_mut_state() = Some(span_context);
        Ok::<_, phper::Error>(object)
    });

    class.add_static_method("create", Visibility::Public, |arguments| {
        let trace_id = arguments[0].expect_z_str()?.to_str()?;
        let span_id = arguments[1].expect_z_str()?.to_str()?;
        let span_context = SpanContext::new(
            TraceId::from_hex(trace_id).map_err(|_| phper::Error::boxed("Invalid trace id format"))?,
            SpanId::from_hex(span_id).map_err(|_| phper::Error::boxed("Invalid trace id format"))?,
            TraceFlags::SAMPLED, //todo
            false,
            TraceState::default(), //todo
        );
        let mut object = SPAN_CONTEXT_CLASS.init_object()?;
        *object.as_mut_state() = Some(span_context);
        Ok::<_, phper::Error>(object)
    });

    class.add_static_method("createFromRemoteParent", Visibility::Public, |arguments| {
        let trace_id = arguments[0].expect_z_str()?.to_str()?;
        let span_id = arguments[1].expect_z_str()?.to_str()?;
        let span_context = SpanContext::new(
            TraceId::from_hex(trace_id).map_err(|_| phper::Error::boxed("Invalid trace id format"))?,
            SpanId::from_hex(span_id).map_err(|_| phper::Error::boxed("Invalid trace id format"))?,
            TraceFlags::SAMPLED, //todo
            true,
            TraceState::default(), //todo
        );
        let mut object = SPAN_CONTEXT_CLASS.init_object()?;
        *object.as_mut_state() = Some(span_context);
        Ok::<_, phper::Error>(object)
    });

    class.add_method("isValid", Visibility::Public, |this, _| {
        let state = this.as_state();
        let is_valid = state.as_ref().expect("kaboom").is_valid();
        Ok::<_, phper::Error>(is_valid)
    });

    class.add_method("getTraceId", Visibility::Public, |this, _| {
        let state = this.as_state();
        let trace_id = state.as_ref().expect("kaboom").trace_id().to_string();
        Ok::<_, phper::Error>(trace_id)
    });

    class.add_method("getSpanId", Visibility::Public, |this, _| {
        let state = this.as_state();
        let span_id = state.as_ref().expect("kaboom").span_id().to_string();
        Ok::<_, phper::Error>(span_id)
    });

    class.add_method("isRemote", Visibility::Public, |this, _| {
        let state = this.as_state();
        let is_remote = state.as_ref().expect("kaboom").is_remote();
        Ok::<_, phper::Error>(is_remote)
    });

    class.add_method("isSampled", Visibility::Public, |this, _| {
        let state = this.as_state();
        let is_sampled = state.as_ref().expect("kaboom").is_sampled();
        Ok::<_, phper::Error>(is_sampled)
    });

    class
}
