use phper::{
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use std::{
    convert::Infallible,
};
use opentelemetry::{
    trace::{
        SpanBuilder,
        Tracer,
    }
};
use opentelemetry_sdk::trace::SdkTracer;
use crate::trace::span_builder::{
    SpanBuilderState,
    SpanBuilderClass,
};
use crate::trace::span_builder_interface::SPAN_BUILDER_INTERFACE_NAME;

pub type TracerClass = StateClass<Option<SdkTracer>>;

const TRACER_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\Tracer";

pub fn make_tracer_class(
    span_builder_class: SpanBuilderClass,
    tracer_interface: Interface,
) -> ClassEntity<Option<SdkTracer>> {
    let mut class =
        ClassEntity::<Option<SdkTracer>>::new_with_default_state_constructor(TRACER_CLASS_NAME);

    class.implements(tracer_interface);
    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("spanBuilder", Visibility::Public, move |this, arguments| {
            // Populated only by `TracerProvider::getTracer()`; anything else is a userland
            // mistake to report, not a reason to panic across the FFI boundary.
            let tracer: &SdkTracer = this.as_state().as_ref().ok_or_else(|| {
                phper::Error::boxed("Tracer was not created by TracerProvider::getTracer()")
            })?;
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();
            let span_builder: SpanBuilder = tracer.span_builder(name);
            let my_span_builder = SpanBuilderState::new(span_builder, tracer.clone());
            let mut object = span_builder_class.init_object()?;
            *object.as_mut_state() = my_span_builder;
            Ok::<_, phper::Error>(object)
        })
        .argument(Argument::new("spanName").with_type_hint(ArgumentTypeHint::String))
        // Declared on the class as well as the interface. An implementation inherits the
        // interface's return type when it declares none, so this is not required for
        // correctness -- but reflection then reports `<none>` on the class, and the class is
        // what most callers actually look at.
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    class
}
