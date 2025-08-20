use phper::{
    classes::{ClassEntity, Visibility},
    functions::{ReturnType},
    types::ReturnTypeHint,
};
use std::{
    cell::RefCell,
    convert::Infallible,
    sync::Arc,
};
use crate::{
    context::{
        storage,
    },
    trace::{
        non_recording_span::{NonRecordingSpanClass},
        span::SpanClass,
    },
};
use opentelemetry::Context;

const LOCAL_ROOT_SPAN_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\LocalRootSpan";

thread_local! {
    static LOCAL_ROOT_SPAN_ID: RefCell<Option<u64>> = RefCell::new(None);
}

pub fn make_local_root_span_class(
    span_class: SpanClass,
    non_recording_span_class: NonRecordingSpanClass,
) -> ClassEntity<()> {
    let mut class =
        ClassEntity::<()>::new_with_default_state_constructor(LOCAL_ROOT_SPAN_CLASS_NAME);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_static_method("current", Visibility::Public, move |_| {
            if let Some(instance_id) = get_local_root_span_instance_id() {
                let mut object = span_class.clone().init_object()?;
                *object.as_mut_state() = None;
                object.set_property("context_id", instance_id as i64);
                object.set_property("is_local_root", true);
                Ok::<_, phper::Error>(object)
            } else {
                tracing::info!("Returning non-recording span");
                let object = non_recording_span_class.clone().init_object()?;
                Ok::<_, phper::Error>(object)
            }
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\API\Span\SpanInterface"))))
    ;

    class
}

pub fn store_local_root_span(context_id: Option<u64>) {
    LOCAL_ROOT_SPAN_ID.with(|cell| *cell.borrow_mut() = context_id);
}

/// Retrieves the stored context instance ID containing the local root span, if it exists.
fn get_local_root_span_instance_id() -> Option<u64> {
    LOCAL_ROOT_SPAN_ID.with(|cell| *cell.borrow())
}

/// Retrieves the context containing the local root span, if it exists.
pub fn get_local_root_span_context() -> Option<Arc<Context>> {
    LOCAL_ROOT_SPAN_ID.with(|cell| {
        if let Some(context_id) = *cell.borrow() {
            storage::get_context_instance(Some(context_id))
        } else {
            None
        }
    })
}

pub fn maybe_remove_local_root_span(context_id: Option<u64>) {
    LOCAL_ROOT_SPAN_ID.with(|cell| {
        let mut borrowed = cell.borrow_mut();
        match (context_id, *borrowed) {
            (Some(id), Some(current_id)) if id == current_id => {
                tracing::debug!("Removing local root span: {}", id);
                *borrowed = None;
                storage::maybe_remove_context_instance(Some(id));
            }
            (None, Some(current_id)) => {
                tracing::debug!("Removing local root span: {}", current_id);
                *borrowed = None;
                storage::maybe_remove_context_instance(Some(current_id));
            }
            _ => {}
        }
    });
}
