use phper::{
    alloc::ToRefOwned,
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::Argument,
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
        scope::ScopeClass,
    },
    trace::{
        span_context::SpanContextClass,
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

    class
        .add_method("setStatus", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("code"))
        .argument(Argument::new("description").optional());

    class
        .add_method("setAttribute", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("setAttributes", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("updateName", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("recordException", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addLink", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addEvent", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

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

    class
        .add_method("storeInContext", Visibility::Public, move |_, _| {
            //TODO
            Ok::<_, phper::Error>(())
        }); //argument ContextInterface, return ContextInterface

    class
        .add_static_method("fromContext", Visibility::Public, move |_| {
            //TODO
            Ok::<_, phper::Error>(())
        });

    class
}
