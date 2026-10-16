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
    trace::{
        SpanBuilder,
        SpanKind,
        Tracer,
    }
};
use opentelemetry_sdk::trace::SdkTracer;
use crate::{
    context::{
        context::get_instance_id,
        context_interface::CONTEXT_INTERFACE_NAME,
        storage,
    },
    trace::{
        span::SpanClass,
    },
    util,
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

/// The state of a SpanBuilder is only populated by `Tracer::spanBuilder()`. An instance
/// obtained any other way (`newInstanceWithoutConstructor()`) has none, which is a userland
/// mistake to report rather than a reason to panic across the FFI boundary.
fn uninitialized() -> phper::Error {
    phper::Error::boxed("SpanBuilder was not created by Tracer::spanBuilder()")
}

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
        let span_builder = state.span_builder.as_ref().ok_or_else(uninitialized)?;
        let key = arguments[0].expect_z_str()?.to_str()?.to_string();
        // `$value` is `mixed` upstream, and forcing it through `expect_z_str` rejected the
        // ints, floats, bools and arrays that `Span::setAttribute` already accepts. Share
        // that conversion rather than keeping a second, narrower one here.
        if let Some(kv) = util::zval_to_key_value(&key, &arguments[1]) {
            let mut attributes = span_builder.attributes.clone().unwrap_or_default();
            attributes.push(kv);
            let new_span_builder = span_builder.clone().with_attributes(attributes);
            state.span_builder = Some(new_span_builder);
        }

        Ok::<_, phper::Error>(this.to_ref_owned())
    })
    .argument(Argument::new("key").with_type_hint(ArgumentTypeHint::String))
    // Previously `.optional()` with no default, so `setAttribute('k')` passed Zend's arity
    // check and then indexed a slot that did not exist -- a panic across the FFI boundary,
    // which aborts the process (SIGABRT). Upstream has no default here either.
    .argument(Argument::new("value").with_type_hint(ArgumentTypeHint::Mixed));

    class
        .add_method("setParent", Visibility::Public, |this, arguments| {
            // `get_instance_id` maps a context_id of 0 to None, which is what
            // `Context::getCurrent()` reports; reading the property directly turned that 0
            // into a parent id that resolves to nothing.
            let context_id = arguments[0]
                .as_z_obj()
                .and_then(get_instance_id)
                .unwrap_or(0);
            this.as_mut_state().parent_context_id = context_id;

            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        // Upstream is `ContextInterface|false|null`. phper has no union type hint, so this
        // accepts null but rejects `false`; see docs/phper-upstream-needs.md. Note that a
        // class-typed argument cannot carry a default value either, so this stays required.
        .argument(
            Argument::new("context")
                .with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                    CONTEXT_INTERFACE_NAME,
                )))
                .allow_null(),
        );

    class
        .add_method("setSpanKind", Visibility::Public, |this, arguments| {
            let state = this.as_mut_state();
            let span_builder = state.span_builder.as_ref().ok_or_else(uninitialized)?;
            let span_kind_int = arguments[0].expect_long()?;
            // These must match `SpanKind`'s constants, which match upstream's: KIND_CLIENT is
            // 1 and KIND_SERVER is 2. They were previously transposed, so every client span
            // was exported as a server span and vice versa.
            let span_kind = match span_kind_int {
                0 => SpanKind::Internal,
                1 => SpanKind::Client,
                2 => SpanKind::Server,
                3 => SpanKind::Producer,
                4 => SpanKind::Consumer,
                _ => {
                    tracing::warn!(
                        "setSpanKind() given unknown span kind {}, defaulting to INTERNAL",
                        span_kind_int
                    );
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
            let span_builder = state.span_builder.as_ref().ok_or_else(uninitialized)?;
            let tracer = state.tracer.as_ref().ok_or_else(uninitialized)?;
            let parent_context = if state.parent_context_id > 0 {
                storage::get_context_instance(Some(state.parent_context_id))
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
