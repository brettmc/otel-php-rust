use phper::{
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::ReturnType,
    types::ReturnTypeHint,
    values::ZVal,
};
use std::sync::Arc;
use std::convert::Infallible;
use crate::context::{
    context::{
        get_instance_id,
        ContextClassEntity,
    },
    context_interface::CONTEXT_INTERFACE_NAME,
    propagation::text_map_propagator_interface::{extract_arguments, inject_arguments},
    storage,
};

/// `API\Trace\Propagation`, matching upstream. The shorter `API\Propagation` would be wrong
/// in both directions: code following upstream's documentation would get a class-not-found
/// error here, and code written against this extension would break on moving to upstream.
const TRACE_CONTEXT_PROPAGATOR_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\Propagation\TraceContextPropagator";

pub type TraceContextPropagatorClass = StateClass<()>;

pub fn make_trace_context_propagator_class(
    text_map_propagator_interface: Interface,
    context_class: &ContextClassEntity,
) -> ClassEntity<()> {
    let mut class = ClassEntity::<()>::new_with_default_state_constructor(
        TRACE_CONTEXT_PROPAGATOR_CLASS_NAME,
    );

    class.implements(text_map_propagator_interface);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    let inject = class
        .add_method("inject", Visibility::Public, |_, arguments| -> phper::Result<()> {
            // Context is the third of three parameters and the first two are optional, so
            // `inject($carrier)` — the ordinary call shape — passes only one argument. The
            // slot has to be reached for rather than indexed: a panic at an `extern "C"`
            // boundary cannot unwind into PHP, so it aborts the process instead of throwing.
            let context_id = arguments
                .get(2)
                .and_then(|arg| arg.as_z_obj())
                .and_then(get_instance_id);
            tracing::debug!("inject() using context_id = {:?}", context_id);

            // `Context::getCurrent()` reports a context_id of 0, which resolves to `None`, so
            // the overwhelmingly common call shape has no stored instance to look up. Fall
            // back to the current context rather than treating that as an error.
            let context = storage::resolve_context(context_id);
            let context_ref = context.as_ref();

            // Carrier gymnastics (PHP array passed by ref)
            let carrier_ref = arguments[0].expect_mut_z_ref()?;
            let carrier_val = carrier_ref.val_mut();
            let mut cloned = carrier_val.clone();

            let zarr = cloned
                .as_mut_z_arr()
                .ok_or_else(|| phper::Error::boxed("Expected carrier to be an array"))?;

            let mut out_map = std::collections::HashMap::<String, String>::new();

            // Use global propagator to inject
            opentelemetry::global::get_text_map_propagator(|prop| {
                prop.inject_context(context_ref, &mut out_map);
            });

            for (k, v) in out_map {
                tracing::debug!(target: "otel::trace::propagation::trace_context_propagator", "inject() inserting {} = {}", k, v);
                zarr.insert(k.as_str(), ZVal::from(v));
            }
            *carrier_val = cloned;
            Ok::<_, phper::Error>(())
        });
    // Taken from the interface rather than spelled out again here. A class whose declaration
    // disagrees with the interface it implements is either a fatal MINIT error or a signature
    // userland cannot rely on, and two copies of a signature drift.
    for argument in inject_arguments() {
        inject.argument(argument);
    }
    inject.return_type(ReturnType::new(ReturnTypeHint::Void));

    let context_ce = context_class.bound_class();
    let extract = class
        .add_method("extract", Visibility::Public, move |_, arguments| {
            // Carrier (input headers)
            let carrier = arguments[0].expect_z_arr()?;

            let mut map = std::collections::HashMap::<String, String>::new();
            for (k, v) in carrier.iter() {
                if let phper::arrays::IterKey::ZStr(k) = k
                    && let Some(zstr) = v.as_z_str()
                    && let Ok(val) = zstr.to_str() {
                    map.insert(k.to_str()?.to_string(), val.to_string());
                }
            }

            // Parent context (optional, and declared `allow_null()`, so an explicit null is
            // a legal way to ask for the current context rather than a type error).
            let context_id = arguments
                .get(2)
                .and_then(|arg| arg.as_z_obj())
                .and_then(get_instance_id);

            let parent_cx = context_id
                .and_then(|id| storage::get_context_instance(Some(id)))
                .unwrap_or_else(|| Arc::new(opentelemetry::Context::current()));

            // Extract new context from headers
            let new_cx = opentelemetry::global::get_text_map_propagator(|prop| {
                prop.extract_with_context(&parent_cx, &map)
            });
            let arc = Arc::new(new_cx);
            let instance_id = storage::store_context_instance(arc.clone());

            // Wrap in PHP context object
            let mut obj = context_ce.init_object()?;
            // Both halves are needed: the `Arc` state, or every method that reaches it --
            // `with()`, `activate()`, `get()` -- reports "not created by a factory method",
            // and the `context_id`, or the object cannot find its PHP-side values.
            *obj.as_mut_state() = Some(arc);
            obj.set_property("context_id", instance_id.unwrap_or(0) as i64);
            Ok::<_, phper::Error>(obj)
        });
    for argument in extract_arguments() {
        extract.argument(argument);
    }
    extract.return_type(ReturnType::new(ReturnTypeHint::ClassEntry(
        CONTEXT_INTERFACE_NAME.to_string(),
    )));

    class
}
