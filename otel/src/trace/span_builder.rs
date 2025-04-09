use phper::{
    classes::{ClassEntity, StateClass, Visibility},
    alloc::ToRefOwned,
    functions::Argument,
    types::ArgumentTypeHint,
};
use std::{
    convert::Infallible,
    sync::Arc,
};
use opentelemetry::{
    Context,
    KeyValue,
    trace::{
        SpanBuilder,
        SpanKind,
        Tracer,
    }
};
use opentelemetry_sdk::trace::SdkTracer;
use crate::{
    context::storage,
    trace::{
        span::SpanClass,
    }
};

pub struct SpanBuilderState {
    span_builder: Option<SpanBuilder>,
    tracer: Option<SdkTracer>,
    parent_context_id: u64,
}
// @see https://github.com/open-telemetry/opentelemetry-rust/issues/2742
impl SpanBuilderState {
    pub fn new(span_builder: SpanBuilder, tracer: SdkTracer) -> Self {
        Self { span_builder: Some(span_builder), tracer: Some(tracer),  parent_context_id: 0 }
    }
    pub fn empty() -> Self {
        Self{ span_builder: None, tracer: None,  parent_context_id: 0 }
    }
}

const SPAN_BUILDER_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\SpanBuilder";

pub type SpanBuilderClass = StateClass<SpanBuilderState>;

pub fn make_span_builder_class(span_class: SpanClass) -> ClassEntity<SpanBuilderState> {
    let mut class = ClassEntity::<SpanBuilderState>::new_with_state_constructor(
        SPAN_BUILDER_CLASS_NAME,
        || {
            SpanBuilderState::empty()
        },
    );

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    //TODO addLink, setAttributes, setStartTimestamp

    class.add_method("setAttribute", Visibility::Public, |this, arguments| {
        let state = this.as_mut_state();
        let span_builder = state.span_builder.as_ref().expect("SpanBuilder not set");
        let name = arguments[0].expect_z_str()?.to_str()?.to_string();
        let value = arguments[1].expect_z_str()?.to_str()?.to_string();
        let mut attributes = span_builder.attributes.clone().unwrap_or_default();
        attributes.push(KeyValue::new(name, value));
        let new_span_builder = span_builder.clone().with_attributes(attributes);
        state.span_builder = Some(new_span_builder);

        Ok::<_, phper::Error>(this.to_ref_owned())
    })
    .argument(Argument::new("key"))
    .argument(Argument::new("value").optional());

    class
        .add_method("setParent", Visibility::Public, |this, arguments| {
            let state = this.as_mut_state();

            let context_obj = arguments[0].expect_mut_z_obj()?;
            let context_id = context_obj.get_property("context_id").as_long().unwrap_or(0);
            state.parent_context_id = context_id as u64;

            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("context").with_type_hint(ArgumentTypeHint::ClassEntry(
            String::from(r"OpenTelemetry\Context\ContextInterface"),
        )));

    class
        .add_method("setSpanKind", Visibility::Public, |this, arguments| {
            let state = this.as_mut_state();
            let span_builder = state.span_builder.as_ref().expect("SpanBuilder not set");
            let span_kind_int = arguments[0].expect_long()?;
            let span_kind = match span_kind_int {
                0 => SpanKind::Internal,
                1 => SpanKind::Server,
                2 => SpanKind::Client,
                3 => SpanKind::Producer,
                4 => SpanKind::Consumer,
                _ => {
                    //log a warning
                    SpanKind::Internal
                },
            };
            let new_span_builder = span_builder.clone().with_kind(span_kind);
            state.span_builder = Some(new_span_builder);

            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("spanKind").with_type_hint(ArgumentTypeHint::Int));

    class
        .add_method("startSpan", Visibility::Public, move |this, _| {
            let state = this.as_state();
            let span_builder = state.span_builder.as_ref().expect("SpanBuilder not set");
            let tracer = state.tracer.as_ref().expect("Tracer not set");
            let parent_context = if state.parent_context_id > 0 {
                storage::get_context_instance(state.parent_context_id)
                    .map(|ctx| {
                        tracing::debug!(
                            "SpanBuilder::Using parent context {} (ref count = {})",
                            state.parent_context_id,
                            Arc::strong_count(&ctx)
                        );
                        (*ctx).clone()
                    })
                    .unwrap_or_else(|| {
                        tracing::warn!(
                            "SpanBuilder::Parent context {} not found, falling back to current()",
                            state.parent_context_id
                        );
                        Context::current()
                    })
            } else {
                tracing::debug!("SpanBuilder::No parent context, using Context::current()");
                Context::current()
            };

            let span = tracer.build_with_context(span_builder.clone(), &parent_context);
            tracing::debug!("SpanBuilder::Starting span");
            let mut object = span_class.init_object()?;
            *object.as_mut_state() = Some(span);
            Ok::<_, phper::Error>(object)
        });

    class
}
