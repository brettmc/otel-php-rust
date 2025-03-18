use crate::trace::plugin::{Handler, HandlerCallbacks, Plugin, SpanDetails};
use opentelemetry::{
    KeyValue,
    trace::SpanRef,
};
use std::sync::Arc;
use phper::values::{ExecuteData, ZVal};
use phper::objects::ZObj;

pub struct TestPlugin {
    handlers: Vec<Arc<dyn Handler + Send + Sync>>,
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
    fn get_handlers(&self) -> Vec<Arc<dyn Handler + Send + Sync>> {
        self.handlers.clone()
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
            pre_observe: Some(Self::pre_callback),
            post_observe: Some(Self::post_callback),
        }
    }
}

impl DemoHandler {
    unsafe extern "C" fn pre_callback(_exec_data: *mut ExecuteData, _span_details: &mut SpanDetails) {
        //println!("DemoHandler::pre_callback");
    }

    unsafe extern "C" fn post_callback(
        _exec_data: *mut ExecuteData,
        _span_ref: &SpanRef,
        _retval: &mut ZVal,
        _exception: Option<&mut ZObj>
    ) {
        //println!("DemoHandler::post_callback");
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
            pre_observe: Some(Self::pre_callback),
            post_observe: Some(Self::post_callback),
        }
    }
}

impl DemoFunctionHandler {
    unsafe extern "C" fn pre_callback(_execute_data: *mut ExecuteData, span_details: &mut SpanDetails) {
        span_details.update_name("i-was-renamed");
        span_details.add_attribute("my-attribute".to_string(), "my-value".to_string());
    }

    unsafe extern "C" fn post_callback(
        _execute_data: *mut ExecuteData,
        span_ref: &SpanRef,
        _retval: &mut ZVal,
        _exception: Option<&mut ZObj>
    ) {
        span_ref.set_attribute(KeyValue::new("post.attribute".to_string(), "post.value".to_string()));
    }
}