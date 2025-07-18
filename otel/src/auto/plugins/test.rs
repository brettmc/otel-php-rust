// A test plugin which implements two handlers:
// - DemoHandler: observes a handful of classes and functions with a pre and post callback
// - DemoFunctionHandler: observes a specific function with a different pre and post callback
use crate::auto::{
    execute_data::{get_default_attributes, get_fqn},
    plugin::{Handler, HandlerList, HandlerSlice, HandlerCallbacks, Plugin},
};
use opentelemetry::{
    KeyValue,
    trace::{Tracer, TracerProvider},
};
use crate::{
    context::storage::{store_guard, take_guard},
    tracer_provider,
};
use std::sync::Arc;
use phper::{
    alloc::ToRefOwned,
    errors::ThrowObject,
    values::{ExecuteData, ZVal},
    objects::ZObj,
};
use opentelemetry::{
    Context,
    trace::TraceContextExt,
};

pub struct TestPlugin {
    handlers: HandlerList,
}

impl TestPlugin {
    pub fn new() -> Self {
        Self {
            handlers: vec![
                Arc::new(DemoHandler),
                Arc::new(DemoFunctionHandler),
            ],
        }
    }
}

impl Plugin for TestPlugin {
    fn is_enabled(&self) -> bool {
        true
    }
    fn get_handlers(&self) -> &HandlerSlice {
        &self.handlers
    }
    fn get_name(&self) -> &str {
        "test"
    }
}

pub struct DemoHandler;

impl Handler for DemoHandler {
    fn get_functions(&self) -> Vec<String> {
        vec![
            "DemoClass::test".to_string(),
            "DemoClass::inner".to_string(),
            "phpversion".to_string(),
        ]
    }
    fn get_interfaces(&self) -> Vec<String> {
        vec![
            "IDemo::foo".to_string(),
            "IDemo::bar".to_string(),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Box::new(|exec_data| unsafe {
                Self::pre_callback(exec_data)
            })),
            post_observe: Some(Box::new(|exec_data, retval, exception| unsafe {
                Self::post_callback(exec_data, retval, exception)
            })),
        }
    }
}

impl DemoHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        let tracer = tracer_provider::get_tracer_provider().tracer("php-auto-instrumentation"); //TODO: store tracer in a static variable
        let attributes = get_default_attributes(&*exec_data);
        let name = get_fqn(&*exec_data);

        let span_builder = tracer.span_builder(name)
            .with_attributes(attributes);
        let span = tracer.build_with_context(span_builder, &Context::current());
        let ctx = Context::current_with_span(span);
        let guard = ctx.attach();
        store_guard(exec_data, guard);
    }

    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        _retval: &mut ZVal,
        _exception: Option<&mut ZObj>
    ) {
        if let Some(_guard) = take_guard(exec_data) {
            //do nothing, _guard will go out of scope at end of function
        } else {
            tracing::warn!("DemoHandler: No context guard found for post callback");
            return;
        }
    }
}

pub struct DemoFunctionHandler;

impl Handler for DemoFunctionHandler {
    fn get_functions(&self) -> Vec<String> {
        vec![
            "demoFunction".to_string(),
        ]
    }
    fn get_interfaces(&self) -> Vec<String> {
        vec![]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Box::new(|exec_data| unsafe {
                Self::pre_callback(exec_data)
            })),
            post_observe: Some(Box::new(|exec_data, retval, exception| unsafe {
                Self::post_callback(exec_data, retval, exception)
            })),
        }
    }
}

impl DemoFunctionHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        let tracer = tracer_provider::get_tracer_provider().tracer("php-auto-instrumentation"); //TODO: store tracer in a static variable
        let mut attributes = get_default_attributes(&*exec_data);
        attributes.push(KeyValue::new("my-attribute", "my-value".to_string()));

        let span_builder = tracer.span_builder("demo-function".to_string())
            .with_attributes(attributes);
        let span = tracer.build_with_context(span_builder, &Context::current());
        let ctx = Context::current_with_span(span);
        let guard = ctx.attach();
        store_guard(exec_data, guard);
    }

    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        _retval: &mut ZVal,
        exception: Option<&mut ZObj>
    ) {
        tracing::debug!("DemoFunctionHandler: post_callback called");
        //get current span
        let context = opentelemetry::Context::current();
        let span_ref = context.span();
        if let Some(exception) = exception {
            if let Ok(throwable) = ThrowObject::new(exception.to_ref_owned()) {
                span_ref.record_error(&throwable);
            }
        }
        span_ref.set_attribute(KeyValue::new("post.attribute".to_string(), "post.value".to_string()));
        if let Some(_guard) = take_guard(exec_data) {
            //do nothing, _guard will go out of scope at end of function
        } else {
            tracing::warn!("DemoFunctionHandler: No context guard found for post callback");
            return;
        }
    }
}