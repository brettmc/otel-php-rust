use phper::{
    alloc::ToRefOwned,
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::Argument,
    types::ArgumentTypeHint,
};
use std::{
    convert::Infallible,
};
use opentelemetry::{
    trace::{
        SpanContext,
    }
};
use opentelemetry_sdk::trace::Span as SdkSpan;
use crate::{
    context::{
        context::ContextClass,
        context_interface::CONTEXT_INTERFACE_NAME,
        scope::ScopeClass,
    },
    trace::{
        span_context::{SpanContextClass, SPAN_CONTEXT_CLASS_NAME},
    },
};

const NON_RECORDING_SPAN_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\NonRecordingSpan";

pub type NonRecordingSpanClass = StateClass<Option<SdkSpan>>; //never Some, but must match other implementations

pub fn make_non_recording_span_class(
    _scope_class: ScopeClass,
    span_context_class: SpanContextClass,
    _context_class: ContextClass,
    span_interface: &Interface,
) -> ClassEntity<Option<SdkSpan>> {
    let mut class =
        ClassEntity::<Option<SdkSpan>>::new_with_default_state_constructor(NON_RECORDING_SPAN_CLASS_NAME);
    let _span_class = class.bound_class();

    class.implements(span_interface.clone());

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("end", Visibility::Public, |_, _| -> phper::Result<()> {
            Ok(())
        });

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
        );

    class
        .add_method("setAttribute", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("key").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("value").with_type_hint(ArgumentTypeHint::Mixed));

    class
        .add_method("setAttributes", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::Iterable));

    class
        .add_method("updateName", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("name").with_type_hint(ArgumentTypeHint::String));

    class
        .add_method("recordException", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("exception").with_type_hint(ArgumentTypeHint::ClassEntry(
            String::from("Throwable"),
        )))
        .argument(
            Argument::new("attributes")
                .with_type_hint(ArgumentTypeHint::Iterable)
                .with_default_value("[]"),
        );

    class
        .add_method("addLink", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(
            String::from(SPAN_CONTEXT_CLASS_NAME),
        )))
        .argument(
            Argument::new("attributes")
                .with_type_hint(ArgumentTypeHint::Iterable)
                .with_default_value("[]"),
        );

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
        );

    class
        .add_method("getContext", Visibility::Public, move |_this, _| {
            let mut object = span_context_class.init_object()?;
            *object.as_mut_state() = Some(SpanContext::empty_context());

            Ok::<_, phper::Error>(object)
        });

    class
        .add_static_method("getCurrent", Visibility::Public, |_| {
            //TODO
            Ok::<_, phper::Error>(())
        });

    class
        .add_method("activate", Visibility::Public, |_, _| {
            //TODO
            Ok::<_, phper::Error>(())
        });

    // Both still return null rather than a Context/Span. The return types are deliberately
    // left undeclared until the bodies are implemented: declaring one here would promise
    // userland a type the handler does not return, and an internal function's return type is
    // not enforced, so the promise would be silently broken rather than caught.
    class
        .add_method("storeInContext", Visibility::Public, move |_, _| {
            //TODO
            Ok::<_, phper::Error>(())
        })
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(
            String::from(CONTEXT_INTERFACE_NAME),
        )));

    class
        .add_static_method("fromContext", Visibility::Public, move |_| {
            //TODO
            Ok::<_, phper::Error>(())
        })
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(
            String::from(CONTEXT_INTERFACE_NAME),
        )));

    class
}
