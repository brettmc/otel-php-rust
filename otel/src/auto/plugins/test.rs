// A test plugin which implements three handlers:
// - DemoHandler: observes a handful of classes and functions with a pre and post callback
// - DemoFunctionHandler: observes a specific function with a different pre and post callback
use crate::{
    auto::{
        execute_data::get_fqn,
        plugin::{Handler, HandlerList, HandlerSlice, HandlerCallbacks, Plugin},
        utils,
    },
    context::storage::take_guard,
    tracer_provider,
};
use opentelemetry::{
    KeyValue,
    trace::{
        TraceContextExt,
        TracerProvider,
    },
};
use std::sync::Arc;
use phper::{
    values::{ExecuteData, ZVal},
    objects::ZObj,
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
                Arc::new(TestClassHandler),
            ],
        }
    }
}

impl Plugin for TestPlugin {
    fn get_handlers(&self) -> &HandlerSlice {
        &self.handlers
    }
    fn get_name(&self) -> &str {
        "test"
    }
    fn request_shutdown(&self) {
        tracing::debug!("Plugin::request_shutdown: {}", self.get_name());
    }
}

pub struct DemoHandler;

impl Handler for DemoHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some("DemoClass".to_string()), "test".to_string()),
            (Some("DemoClass".to_string()), "inner".to_string()),
            (None, "phpversion".to_string()),
            (Some("IDemo".to_string()), "foo".to_string()),
            (Some("IDemo".to_string()), "bar".to_string()),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Box::new(|exec_data| unsafe {
                Self::pre_callback(exec_data)
            })),
            post_observe: Some(Box::new(|exec_data, exception| unsafe {
                Self::post_callback(exec_data, exception)
            })),
        }
    }
}

impl DemoHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        let tracer = tracer_provider::get_tracer_provider().tracer("php.otel.auto.test");
        let exec_data_ref = unsafe { &*exec_data };
        let span_name = get_fqn(exec_data_ref).unwrap_or_default();

        utils::start_and_activate_span(tracer, &span_name, vec![], exec_data, opentelemetry::trace::SpanKind::Internal);
    }

    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
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
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (None, "demoFunction".to_string()),
        ]
    }

    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Box::new(|exec_data| unsafe {
                Self::pre_callback(exec_data)
            })),
            post_observe: Some(Box::new(|exec_data, exception| unsafe {
                Self::post_callback(exec_data, exception)
            })),
        }
    }
}

impl DemoFunctionHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        let tracer = tracer_provider::get_tracer_provider().tracer("php.otel.auto.test");
        let mut attributes = vec![];
        attributes.push(KeyValue::new("my-attribute", "my-value".to_string()));
        let span_name = "demo-function";

        utils::start_and_activate_span(tracer, &span_name, attributes, exec_data, opentelemetry::trace::SpanKind::Internal);
    }

    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        exception: Option<&mut ZObj>
    ) {
        tracing::debug!("DemoFunctionHandler: post_callback called");
        //get current span
        let context = opentelemetry::Context::current();
        let span_ref = context.span();
        if let Some(exception) = exception {
            utils::record_exception(&opentelemetry::Context::current(), exception);
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

pub struct TestClassHandler;
impl Handler for TestClassHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some(r"OpenTelemetry\Test\ITestClass".to_string()), "getString".to_string()),
            (Some(r"OpenTelemetry\Test\ITestClass".to_string()), "throwException".to_string()),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Box::new(|exec_data| unsafe {
                Self::pre_callback(exec_data)
            })),
            post_observe: Some(Box::new(|exec_data, exception| unsafe {
                Self::post_callback(exec_data, exception)
            })),
        }
    }
}

impl TestClassHandler {
    unsafe extern "C" fn pre_callback(_exec_data: *mut ExecuteData) {
        tracing::debug!("TestClassHandler: pre_callback called");
    }

    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        exception: Option<&mut ZObj>
    ) {
        let exec_data_ref = unsafe { &mut *exec_data };
        let retval = exec_data_ref.get_return_value();
        tracing::debug!("TestClassHandler: post_callback called");
        tracing::debug!("retval type: {:?}", retval.unwrap().get_type_info());
        tracing::debug!("exception: {:?}", exception);
    }
}