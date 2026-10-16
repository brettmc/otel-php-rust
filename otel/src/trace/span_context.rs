use phper::{
    classes::{ClassEntity, ClassEntry, Interface, StateClass, Visibility},
    functions::{Argument, ReturnType},
    objects::ZObj,
    types::{ArgumentTypeHint, ReturnTypeHint},
    values::ZVal,
};
use std::{
    convert::Infallible,
    sync::LazyLock,
};
use crate::trace::span_context_interface::SPAN_CONTEXT_INTERFACE_NAME;
use opentelemetry::trace::{
    SpanContext,
    TraceFlags,
    TraceState,
    SpanId,
    TraceId,
};

pub const SPAN_CONTEXT_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\SpanContext";

pub type SpanContextClass = StateClass<Option<SpanContext>>;

pub fn make_span_context_class(
    span_context_interface: &Interface,
) -> ClassEntity<Option<SpanContext>> {
    let mut class =
        ClassEntity::<Option<SpanContext>>::new_with_default_state_constructor(SPAN_CONTEXT_CLASS_NAME);
    let span_context_class = class.bound_class();

    class.implements(span_context_interface.clone());

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    let span_context_class_invalid = span_context_class.clone();
    class
        .add_static_method("getInvalid", Visibility::Public, move |_arguments| {
            let span_context = SpanContext::empty_context();
            let mut object = span_context_class_invalid.clone().init_object()?;
            *object.as_mut_state() = Some(span_context);
            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(SPAN_CONTEXT_INTERFACE_NAME))));

    let span_context_class_create = span_context_class.clone();
    class
        .add_static_method("create", Visibility::Public, move |arguments| {
            let span_context = span_context_from_arguments(arguments, false)?;
            let mut object = span_context_class_create.init_object()?;
            *object.as_mut_state() = Some(span_context);
            Ok::<_, phper::Error>(object)
        })
        .argument(Argument::new("traceId").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("spanId").with_type_hint(ArgumentTypeHint::String))
        .argument(
            Argument::new("traceFlags")
                .with_type_hint(ArgumentTypeHint::Int)
                .with_default_value("0"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(SPAN_CONTEXT_INTERFACE_NAME))));

    let span_context_class_create_from_remote_parent = span_context_class.clone();
    class
        .add_static_method("createFromRemoteParent", Visibility::Public, move |arguments| {
            let span_context = span_context_from_arguments(arguments, true)?;
            let mut object = span_context_class_create_from_remote_parent.clone().init_object()?;
            *object.as_mut_state() = Some(span_context);
            Ok::<_, phper::Error>(object)
        })
        .argument(Argument::new("traceId").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("spanId").with_type_hint(ArgumentTypeHint::String))
        .argument(
            Argument::new("traceFlags")
                .with_type_hint(ArgumentTypeHint::Int)
                .with_default_value("0"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(SPAN_CONTEXT_INTERFACE_NAME))));

    // An unpopulated state means we never had a span context to wrap (for example
    // `getContext()` on a Span with neither an SdkSpan nor a stored context). Reporting
    // the invalid context is what the spec says such a context looks like, and avoids
    // panicking across the FFI boundary into PHP.
    class
        .add_method("isValid", Visibility::Public, |this, _| {
            let is_valid = span_context_or_invalid(this.as_state()).is_valid();
            Ok::<_, phper::Error>(is_valid)
        })
        .return_type(ReturnType::new(ReturnTypeHint::Bool));

    class
        .add_method("getTraceId", Visibility::Public, |this, _| {
            let trace_id = span_context_or_invalid(this.as_state()).trace_id().to_string();
            Ok::<_, phper::Error>(trace_id)
        })
        .return_type(ReturnType::new(ReturnTypeHint::String));

    class
        .add_method("getSpanId", Visibility::Public, |this, _| {
            let span_id = span_context_or_invalid(this.as_state()).span_id().to_string();
            Ok::<_, phper::Error>(span_id)
        })
        .return_type(ReturnType::new(ReturnTypeHint::String));

    // The raw flags byte, alongside `isSampled()`: a caller needs it to build a `traceparent`
    // by hand, or to read a flag `isSampled()` does not cover.
    class
        .add_method("getTraceFlags", Visibility::Public, |this, _| {
            let flags = span_context_or_invalid(this.as_state()).trace_flags().to_u8();
            Ok::<_, phper::Error>(flags as i64)
        })
        .return_type(ReturnType::new(ReturnTypeHint::Int));

    class
        .add_method("isRemote", Visibility::Public, |this, _| {
            let is_remote = span_context_or_invalid(this.as_state()).is_remote();
            Ok::<_, phper::Error>(is_remote)
        })
        .return_type(ReturnType::new(ReturnTypeHint::Bool));

    class
        .add_method("isSampled", Visibility::Public, |this, _| {
            let is_sampled = span_context_or_invalid(this.as_state()).is_sampled();
            Ok::<_, phper::Error>(is_sampled)
        })
        .return_type(ReturnType::new(ReturnTypeHint::Bool));

    class
}

/// Build a `SpanContext` from the arguments shared by `create` and `createFromRemoteParent`.
///
/// The caller's flags are honoured rather than assumed, defaulting to `TraceFlags::DEFAULT`
/// (0, not sampled) as upstream does. Assuming `SAMPLED` here would make an explicitly
/// unsampled context report itself sampled, and for a remote parent it would override the
/// incoming sampling decision — which is the main reason to call `createFromRemoteParent`.
///
/// Upstream also takes a fourth `?TraceStateInterface $traceState = null`, which is not
/// declared here: there is no `TraceState` class to accept, so trace state is defaulted.
fn span_context_from_arguments(
    arguments: &mut [ZVal],
    remote: bool,
) -> phper::Result<SpanContext> {
    let trace_id = arguments[0].expect_z_str()?.to_str()?;
    let span_id = arguments[1].expect_z_str()?.to_str()?;
    // Declared optional with a default, so a two-argument call leaves no slot to index.
    let trace_flags = match arguments.get(2) {
        Some(zval) => TraceFlags::new(zval.expect_long()? as u8),
        None => TraceFlags::NOT_SAMPLED,
    };

    Ok(SpanContext::new(
        TraceId::from_hex(trace_id)
            .map_err(|_| phper::Error::boxed("Invalid trace id format"))?,
        SpanId::from_hex(span_id).map_err(|_| phper::Error::boxed("Invalid span id format"))?,
        trace_flags,
        remote,
        TraceState::default(),
    ))
}

/// Read a `SpanContext` out of any PHP `SpanContextInterface`.
///
/// Callers accept the interface, so the object may be a userland implementation rather than
/// our `SpanContext` class. Two things follow.
///
/// `as_state_obj` is only sound for a class this extension defined -- on anything else it
/// reinterprets unrelated memory as our state, which is a crash rather than an exception --
/// so the class is checked before the cast. An unguarded cast is only safe where the argument
/// is typed as the concrete class, and every caller here accepts the interface.
///
/// And a foreign object is not an error: upstream accepts any `SpanContextInterface`, and the
/// interface exposes everything a `SpanContext` holds, so one is rebuilt by calling the
/// accessors. That costs a handful of PHP method calls, which is why our own class still takes
/// the direct path.
pub fn span_context_from_object(object: &mut ZObj) -> phper::Result<SpanContext> {
    let ours = ClassEntry::from_globals(SPAN_CONTEXT_CLASS_NAME)
        .is_ok_and(|expected| object.get_class().is_instance_of(expected));
    if ours {
        let state_obj = unsafe { object.as_state_obj::<Option<SpanContext>>() };
        return state_obj
            .as_state()
            .clone()
            .ok_or_else(|| phper::Error::boxed("SpanContext object has no state"));
    }

    // Trace state is not read back: there is no `TraceState` class for a userland
    // implementation to return one from, so `getTraceState()` could only ever answer null.
    // `span_context_from_arguments` defaults it for the same reason.
    let trace_id = object.call("getTraceId", [])?;
    let trace_id = trace_id.expect_z_str()?.to_str()?;
    let span_id = object.call("getSpanId", [])?;
    let span_id = span_id.expect_z_str()?.to_str()?;
    let trace_flags = object.call("getTraceFlags", [])?.expect_long()?;
    let is_remote = object.call("isRemote", [])?.expect_bool()?;

    Ok(SpanContext::new(
        TraceId::from_hex(trace_id).map_err(|_| phper::Error::boxed("Invalid trace id format"))?,
        SpanId::from_hex(span_id).map_err(|_| phper::Error::boxed("Invalid span id format"))?,
        TraceFlags::new(trace_flags as u8),
        is_remote,
        TraceState::default(),
    ))
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
