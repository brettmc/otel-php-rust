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
    trace::tracer_provider,
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
                Arc::new(DemoHelloHandler),
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
    fn get_targets(&self) -> Vec<(Option<&'static str>, &'static str)> {
        vec![
            (Some("DemoClass"), "test"),
            (Some("DemoClass"), "inner"),
            (None, "phpversion"),
            (Some("IDemo"), "foo"),
            (Some("IDemo"), "bar"),
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
        let tracer = tracer_provider::get_tracer_provider().tracer("php.otel.auto.test");
        let exec_data_ref = unsafe { &*exec_data };
        let span_name = get_fqn(exec_data_ref).unwrap_or_default();

        utils::start_and_activate_span(tracer, &span_name, vec![], exec_data, opentelemetry::trace::SpanKind::Internal);
    }

    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        _retval: &mut ZVal,
        _exception: Option<&mut ZObj>
    ) {
        take_guard(exec_data);
    }
}

pub struct DemoHelloHandler;

impl Handler for DemoHelloHandler {
    fn get_targets(&self) -> Vec<(Option<&'static str>, &'static str)> {
        vec![
            (Some("DemoClass"), "hello"),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: None,
            post_observe: Some(Box::new(|exec_data, retval, exception| unsafe {
                Self::post_callback(exec_data, retval, exception)
            })),
        }
    }
}

impl DemoHelloHandler {
    unsafe extern "C" fn post_callback(
        _exec_data: *mut ExecuteData,
        retval: &mut ZVal,
        _exception: Option<&mut ZObj>
    ) {
        tracing::debug!("DemoFunctionHandler: post_callback called");
        // Print info about the return value
        tracing::debug!("Return value type: {:?}", retval.get_type_info());
        tracing::debug!("Return value (debug): {:?}", retval);

        *retval = ZVal::from("goodbye");
        tracing::debug!("Return value mutated to: {:?}", retval);
    }
}

pub struct DemoFunctionHandler;

impl Handler for DemoFunctionHandler {
    fn get_targets(&self) -> Vec<(Option<&'static str>, &'static str)> {
        vec![
            (None, "demoFunction"),
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
        _retval: &mut ZVal,
        exception: Option<&mut ZObj>
    ) {
        tracing::debug!("DemoFunctionHandler: post_callback called");
        let _guard = take_guard(exec_data);
        //get current span
        let context = opentelemetry::Context::current();
        let span_ref = context.span();
        if let Some(exception) = exception {
            utils::record_exception(&opentelemetry::Context::current(), exception);
        }
        span_ref.set_attribute(KeyValue::new("post.attribute".to_string(), "post.value".to_string()));
    }
}

pub struct TestClassHandler;
impl Handler for TestClassHandler {
    fn get_targets(&self) -> Vec<(Option<&'static str>, &'static str)> {
        vec![
            (Some(r"OpenTelemetry\Test\ITestClass"), "getString"),
            (Some(r"OpenTelemetry\Test\ITestClass"), "throwException"),
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

impl TestClassHandler {
    unsafe extern "C" fn pre_callback(_exec_data: *mut ExecuteData) {
        tracing::debug!("TestClassHandler: pre_callback called");
    }

    unsafe extern "C" fn post_callback(
        _exec_data: *mut ExecuteData,
        retval: &mut ZVal,
        exception: Option<&mut ZObj>
    ) {
        tracing::debug!("TestClassHandler: post_callback called");
        tracing::debug!("retval type: {:?}", retval.get_type_info());
        tracing::debug!("exception: {:?}", exception);
    }
}