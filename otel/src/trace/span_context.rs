use phper::{
    classes::{ClassEntity, StateClass, Visibility},
};
use std::{
    convert::Infallible,
    sync::LazyLock,
};
use opentelemetry::trace::{
    SpanContext,
    TraceFlags,
    TraceState,
    SpanId,
    TraceId,
};

const SPAN_CONTEXT_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\SpanContext";

pub type SpanContextClass = StateClass<Option<SpanContext>>;

pub fn make_span_context_class() -> ClassEntity<Option<SpanContext>> {
    let mut class =
        ClassEntity::<Option<SpanContext>>::new_with_default_state_constructor(SPAN_CONTEXT_CLASS_NAME);
    let span_context_class = class.bound_class();

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    let span_context_class_invalid = span_context_class.clone();
    class.add_static_method("getInvalid", Visibility::Public, move |_arguments| {
        let span_context = SpanContext::empty_context();
        let mut object = span_context_class_invalid.clone().init_object()?;
        *object.as_mut_state() = Some(span_context);
        Ok::<_, phper::Error>(object)
    });

    let span_context_class_create = span_context_class.clone();
    class.add_static_method("create", Visibility::Public, move |arguments| {
        let trace_id = arguments[0].expect_z_str()?.to_str()?;
        let span_id = arguments[1].expect_z_str()?.to_str()?;
        let span_context = SpanContext::new(
            TraceId::from_hex(trace_id).map_err(|_| phper::Error::boxed("Invalid trace id format"))?,
            SpanId::from_hex(span_id).map_err(|_| phper::Error::boxed("Invalid trace id format"))?,
            TraceFlags::SAMPLED, //todo
            false,
            TraceState::default(), //todo
        );
        let mut object = span_context_class_create.init_object()?;
        *object.as_mut_state() = Some(span_context);
        Ok::<_, phper::Error>(object)
    });

    let span_context_class_create_from_remote_parent = span_context_class.clone();
    class.add_static_method("createFromRemoteParent", Visibility::Public, move |arguments| {
        let trace_id = arguments[0].expect_z_str()?.to_str()?;
        let span_id = arguments[1].expect_z_str()?.to_str()?;
        let span_context = SpanContext::new(
            TraceId::from_hex(trace_id).map_err(|_| phper::Error::boxed("Invalid trace id format"))?,
            SpanId::from_hex(span_id).map_err(|_| phper::Error::boxed("Invalid trace id format"))?,
            TraceFlags::SAMPLED, //todo
            true,
            TraceState::default(), //todo
        );
        let mut object = span_context_class_create_from_remote_parent.clone().init_object()?;
        *object.as_mut_state() = Some(span_context);
        Ok::<_, phper::Error>(object)
    });

    // An unpopulated state means we never had a span context to wrap (for example
    // `getContext()` on a Span with neither an SdkSpan nor a stored context). Reporting
    // the invalid context is what the spec says such a context looks like, and avoids
    // panicking across the FFI boundary into PHP.
    class.add_method("isValid", Visibility::Public, |this, _| {
        let is_valid = span_context_or_invalid(this.as_state()).is_valid();
        Ok::<_, phper::Error>(is_valid)
    });

    class.add_method("getTraceId", Visibility::Public, |this, _| {
        let trace_id = span_context_or_invalid(this.as_state()).trace_id().to_string();
        Ok::<_, phper::Error>(trace_id)
    });

    class.add_method("getSpanId", Visibility::Public, |this, _| {
        let span_id = span_context_or_invalid(this.as_state()).span_id().to_string();
        Ok::<_, phper::Error>(span_id)
    });

    class.add_method("isRemote", Visibility::Public, |this, _| {
        let is_remote = span_context_or_invalid(this.as_state()).is_remote();
        Ok::<_, phper::Error>(is_remote)
    });

    class.add_method("isSampled", Visibility::Public, |this, _| {
        let is_sampled = span_context_or_invalid(this.as_state()).is_sampled();
        Ok::<_, phper::Error>(is_sampled)
    });

    class
}

/// Borrow the wrapped `SpanContext`, falling back to the invalid context if this object's
/// state was never populated.
fn span_context_or_invalid(state: &Option<SpanContext>) -> &SpanContext {
    static INVALID: LazyLock<SpanContext> = LazyLock::new(SpanContext::empty_context);
    match state {
        Some(span_context) => span_context,
        None => {
            tracing::debug!("SpanContext::no state stored, reporting the invalid span context");
            &INVALID
        }
    }
}
