use crate::{
    context::{
        context::{ContextClass, get_instance_id},
        context_interface::CONTEXT_INTERFACE_NAME,
        scope::ScopeClass,
        scope_interface::SCOPE_INTERFACE_NAME,
        storage,
    },
    trace::{
        span_context::SpanContextClass, span_context_interface::SPAN_CONTEXT_INTERFACE_NAME,
        span_interface::SPAN_INTERFACE_NAME,
    },
};
use opentelemetry::trace::{SpanContext, TraceContextExt};
use opentelemetry_sdk::trace::Span as SdkSpan;
use phper::{
    alloc::ToRefOwned,
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::{Argument, ReturnType},
    objects::ZObj,
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use std::{convert::Infallible, sync::Arc};

const NON_RECORDING_SPAN_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\NonRecordingSpan";

pub type NonRecordingSpanClass = StateClass<Option<SdkSpan>>; //never Some, but must match other implementations

pub fn make_non_recording_span_class(
    scope_class: ScopeClass,
    span_context_class: SpanContextClass,
    context_class: ContextClass,
    span_interface: &Interface,
) -> ClassEntity<Option<SdkSpan>> {
    let mut class = ClassEntity::<Option<SdkSpan>>::new_with_default_state_constructor(
        NON_RECORDING_SPAN_CLASS_NAME,
    );
    let span_class = class.bound_class();

    class.implements(span_interface.clone());

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("end", Visibility::Public, |_, _| -> phper::Result<()> {
            Ok(())
        })
        // Upstream's `end(?int $endEpochNanos = null)`. Nothing is recorded, so the timestamp
        // is accepted and dropped -- but the argument has to be declared, or a caller holding
        // a `SpanInterface` cannot pass it to whichever implementation it actually has.
        .argument(
            Argument::new("endEpochNanos")
                .with_type_hint(ArgumentTypeHint::Int)
                .allow_null()
                .with_default_value("null"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::Void));

    class
        // Constant false: a non-recording span is by definition not recording.
        .add_method("isRecording", Visibility::Public, |_, _| {
            Ok::<_, phper::Error>(false)
        })
        .return_type(ReturnType::new(ReturnTypeHint::Bool));

    // Every method below is a no-op, but the signatures must match `Span`'s: both implement
    // `SpanInterface`, and userland holding one does not know which it has. An arity or type
    // that differs between the two makes the same call legal against one and a TypeError
    // against the other.

    class
        .add_method("setStatus", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("code").with_type_hint(ArgumentTypeHint::String))
        .argument(
            Argument::new("description")
                .with_type_hint(ArgumentTypeHint::String)
                .allow_null()
                .with_default_value("null"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    class
        .add_method("setAttribute", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("key").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("value").with_type_hint(ArgumentTypeHint::Mixed))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    class
        .add_method("setAttributes", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::Iterable))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    class
        .add_method("updateName", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("name").with_type_hint(ArgumentTypeHint::String))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    class
        .add_method("recordException", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(
            Argument::new("exception")
                .with_type_hint(ArgumentTypeHint::ClassEntry(String::from("Throwable"))),
        )
        .argument(
            Argument::new("attributes")
                .with_type_hint(ArgumentTypeHint::Iterable)
                .with_default_value("[]"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    class
        .add_method("addLink", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(
            Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                SPAN_CONTEXT_INTERFACE_NAME,
            ))),
        )
        .argument(
            Argument::new("attributes")
                .with_type_hint(ArgumentTypeHint::Iterable)
                .with_default_value("[]"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    class
        .add_method("addEvent", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("name").with_type_hint(ArgumentTypeHint::String))
        .argument(
            Argument::new("attributes")
                .with_type_hint(ArgumentTypeHint::Iterable)
                .with_default_value("[]"),
        )
        .argument(
            Argument::new("timestamp")
                .with_type_hint(ArgumentTypeHint::Int)
                .allow_null()
                .with_default_value("null"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    class
        .add_method("getContext", Visibility::Public, move |_this, _| {
            let mut object = span_context_class.init_object()?;
            *object.as_mut_state() = Some(SpanContext::empty_context());

            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_CONTEXT_INTERFACE_NAME,
        ))));

    // A non-recording span has nothing to record, but it still has to *be* a span: the four
    // methods below each return a real object rather than Rust `()`. `getCurrent()` and
    // `fromContext()` are the documented way to reach the ambient span, and a null from
    // `activate()` would make `$span->activate()->detach()` a fatal error.

    class
        // The current span is only ever a NonRecordingSpan when the ambient context holds no
        // valid span, which is exactly when the invalid context is the right answer.
        .add_static_method("getCurrent", Visibility::Public, {
            let span_class = span_class.clone();
            move |_| {
                let object = span_class.init_object()?;
                Ok::<_, phper::Error>(object)
            }
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    class
        // Attaching a non-recording span still has to produce a working Scope: instrumentation
        // calls `activate()` unconditionally and detaches in a `finally`. The attached context
        // carries the invalid span context, so a child started inside the scope correctly sees
        // no valid parent.
        .add_method("activate", Visibility::Public, {
            let scope_class = scope_class.clone();
            move |_this, _| {
                let ctx = storage::current_context()
                    .with_span(opentelemetry::trace::noop::NoopSpan::DEFAULT);
                let instance_id = storage::store_context_instance(Arc::new(ctx));
                storage::attach_context(instance_id).map_err(phper::Error::boxed)?;

                let mut object = scope_class.init_object()?;
                object.set_property("context_id", instance_id.unwrap_or(0) as i64);
                Ok::<_, phper::Error>(object)
            }
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SCOPE_INTERFACE_NAME,
        ))));

    class
        .add_method("storeInContext", Visibility::Public, {
            let context_class = context_class.clone();
            move |_this, arguments| {
                let context_obj: &mut ZObj = arguments[0].expect_mut_z_obj()?;
                let parent_id = get_instance_id(context_obj);
                let parent = storage::resolve_context(parent_id);
                let arc_ctx =
                    Arc::new(parent.with_span(opentelemetry::trace::noop::NoopSpan::DEFAULT));
                let instance_id = storage::store_context_instance(arc_ctx.clone());
                storage::inherit_context_values(parent_id, instance_id);

                let mut object = context_class.init_object()?;
                *object.as_mut_state() = Some(arc_ctx);
                object.set_property("context_id", instance_id.unwrap_or(0) as i64);
                Ok::<_, phper::Error>(object)
            }
        })
        .argument(
            Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                CONTEXT_INTERFACE_NAME,
            ))),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_INTERFACE_NAME,
        ))));

    class
        // Declared because `SpanInterface` requires it, but this class can only ever stand for
        // the non-recording span, so the context is accepted and ignored rather than consulted.
        // Callers wanting the span actually in a context want `Span::fromContext()`.
        .add_static_method("fromContext", Visibility::Public, {
            let span_class = span_class.clone();
            move |_arguments| {
                let object = span_class.init_object()?;
                Ok::<_, phper::Error>(object)
            }
        })
        .argument(
            Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                CONTEXT_INTERFACE_NAME,
            ))),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    class
}
