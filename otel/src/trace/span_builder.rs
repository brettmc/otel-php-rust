use crate::{
    context::{context::get_instance_id, context_interface::CONTEXT_INTERFACE_NAME, storage},
    trace::{
        span::SpanClass, span_builder_interface::SPAN_BUILDER_INTERFACE_NAME,
        span_context::span_context_from_object,
        span_context_interface::SPAN_CONTEXT_INTERFACE_NAME, span_interface::SPAN_INTERFACE_NAME,
    },
    util,
};
use opentelemetry::{
    Context,
    trace::{Link, SpanBuilder, SpanKind, Tracer},
};
use opentelemetry_sdk::trace::SdkTracer;
use phper::{
    alloc::ToRefOwned,
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use std::{
    convert::Infallible,
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

/// What `setParent()` was given, resolved to a context only in `startSpan()`, since the
/// current context can change in between.
enum Parent {
    /// Unset, or `setParent(null)`.
    Current,
    /// `setParent(false)`: no parent, so the span starts a new trace.
    Root,
    /// A context stored under this id.
    Stored(u64),
}

pub struct SpanBuilderState {
    span_builder: Option<SpanBuilder>,
    tracer: Option<SdkTracer>,
    parent: Parent,
}
// @see https://github.com/open-telemetry/opentelemetry-rust/issues/2742
impl SpanBuilderState {
    pub fn new(span_builder: SpanBuilder, tracer: SdkTracer) -> Self {
        Self {
            span_builder: Some(span_builder),
            tracer: Some(tracer),
            parent: Parent::Current,
        }
    }
    pub fn empty() -> Self {
        Self {
            span_builder: None,
            tracer: None,
            parent: Parent::Current,
        }
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

pub fn make_span_builder_class(
    span_class: SpanClass,
    span_builder_interface: &Interface,
) -> ClassEntity<SpanBuilderState> {
    let mut class = ClassEntity::<SpanBuilderState>::new_with_state_constructor(
        SPAN_BUILDER_CLASS_NAME,
        SpanBuilderState::empty,
    );

    class.implements(span_builder_interface.clone());

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("addLink", Visibility::Public, |this, arguments| {
            let span_context = span_context_from_object(arguments[0].expect_mut_z_obj()?)?;
            // `with_links` drops invalid contexts itself, but reporting the no-op here is
            // more useful than silently discarding the call.
            if !span_context.is_valid() {
                tracing::debug!("SpanBuilder::addLink given an invalid span context, ignoring");
                return Ok::<_, phper::Error>(this.to_ref_owned());
            }
            let attributes = match arguments.get(1) {
                Some(zval) => util::zval_arr_to_key_value_vec(zval.expect_z_arr()?.to_owned()),
                None => vec![],
            };

            let state = this.as_mut_state();
            let span_builder = state.span_builder.as_ref().ok_or_else(uninitialized)?;
            let mut links = span_builder.links.clone().unwrap_or_default();
            links.push(Link::new(span_context, attributes, 0));
            state.span_builder = Some(span_builder.clone().with_links(links));

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
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    class
        // Adds to what is already there rather than replacing it, so repeated calls and a
        // preceding `setAttribute` both accumulate -- the same semantics `setAttribute` has
        // here and that upstream's `array_merge` into `$attributes` gives.
        .add_method("setAttributes", Visibility::Public, |this, arguments| {
            let new_attributes =
                util::zval_arr_to_key_value_vec(arguments[0].expect_z_arr()?.to_owned());

            let state = this.as_mut_state();
            let span_builder = state.span_builder.as_ref().ok_or_else(uninitialized)?;
            let mut attributes = span_builder.attributes.clone().unwrap_or_default();
            attributes.extend(new_attributes);
            state.span_builder = Some(span_builder.clone().with_attributes(attributes));

            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::Iterable))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    class
        .add_method(
            "setStartTimestamp",
            Visibility::Public,
            |this, arguments| {
                let nanos = arguments[0].expect_long()?;
                // Upstream takes nanoseconds since the epoch; `SystemTime` cannot represent a
                // negative one, and a caller passing one has a bug rather than a pre-1970 span.
                let nanos: u64 = nanos.try_into().map_err(|_| {
                    phper::Error::boxed("setStartTimestamp() requires a non-negative timestamp")
                })?;
                let start_time = UNIX_EPOCH + Duration::from_nanos(nanos);

                let state = this.as_mut_state();
                let span_builder = state.span_builder.as_ref().ok_or_else(uninitialized)?;
                state.span_builder = Some(span_builder.clone().with_start_time(start_time));

                Ok::<_, phper::Error>(this.to_ref_owned())
            },
        )
        .argument(Argument::new("timestampNanos").with_type_hint(ArgumentTypeHint::Int))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    class
        .add_method("setAttribute", Visibility::Public, |this, arguments| {
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
        // Required, as upstream has it. An `.optional()` here would let `setAttribute('k')`
        // past Zend's arity check and into a slot the handler indexes, panicking across the
        // FFI boundary -- which aborts the process rather than throwing.
        .argument(Argument::new("value").with_type_hint(ArgumentTypeHint::Mixed))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    class
        .add_method("setParent", Visibility::Public, |this, arguments| {
            // Matched on the value rather than trusting the declared type: from 8.0 the engine
            // does not check an internal function's argument types, so anything can arrive.
            // `get_instance_id` rather than the `context_id` property directly: it maps 0 to
            // None, and 0 is what `Context::getCurrent()` reports. Taken literally that 0 is
            // a parent id resolving to no stored context.
            let parent = match arguments.get(0) {
                Some(zval) if zval.as_bool() == Some(false) => Parent::Root,
                Some(zval) => match zval.as_z_obj().and_then(get_instance_id) {
                    Some(context_id) => Parent::Stored(context_id),
                    None => Parent::Current,
                },
                None => Parent::Current,
            };
            this.as_mut_state().parent = parent;

            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        // Required, as upstream has it: there is no default.
        .argument(
            Argument::new("context").with_type_hint(ArgumentTypeHint::Union(vec![
                ArgumentTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME)),
                ArgumentTypeHint::False,
                ArgumentTypeHint::Null,
            ])),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    class
        .add_method("setSpanKind", Visibility::Public, |this, arguments| {
            let state = this.as_mut_state();
            let span_builder = state.span_builder.as_ref().ok_or_else(uninitialized)?;
            let span_kind_int = arguments[0].expect_long()?;
            // These must match `SpanKind`'s constants, which match upstream's: KIND_CLIENT is
            // 1 and KIND_SERVER is 2. Transposing the two is silent -- spans still export,
            // under the wrong kind -- so the mapping is pinned by a round-trip test.
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
                }
            };
            let new_span_builder = span_builder.clone().with_kind(span_kind);
            state.span_builder = Some(new_span_builder);

            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("spanKind").with_type_hint(ArgumentTypeHint::Int))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_BUILDER_INTERFACE_NAME,
        ))));

    class
        .add_method("startSpan", Visibility::Public, move |this, _| {
            let state = this.as_state();
            let span_builder = state.span_builder.as_ref().ok_or_else(uninitialized)?;
            let tracer = state.tracer.as_ref().ok_or_else(uninitialized)?;
            let parent_context = match state.parent {
                Parent::Stored(context_id) => storage::get_context_instance(Some(context_id))
                    .map(|ctx| {
                        tracing::debug!(
                            "SpanBuilder::Using parent context {} (ref count = {})",
                            context_id,
                            Arc::strong_count(&ctx)
                        );
                        (*ctx).clone()
                    })
                    .unwrap_or_else(|| {
                        tracing::warn!(
                            "SpanBuilder::Parent context {} not found, falling back to current()",
                            context_id
                        );
                        Context::current()
                    }),
                Parent::Root => {
                    tracing::debug!("SpanBuilder::Parent is false, starting a new trace");
                    Context::new()
                }
                Parent::Current => {
                    tracing::debug!("SpanBuilder::No parent context, using Context::current()");
                    Context::current()
                }
            };

            let span = tracer.build_with_context(span_builder.clone(), &parent_context);
            tracing::debug!("SpanBuilder::Starting span");
            let mut object = span_class.init_object()?;
            *object.as_mut_state() = Some(span);
            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SPAN_INTERFACE_NAME,
        ))));

    class
}
