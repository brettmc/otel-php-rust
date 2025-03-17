use crate::trace::plugin::{Handler, HandlerCallbacks, Plugin, SpanDetails};
use opentelemetry::{
    KeyValue,
    trace::SpanRef,
};
use std::sync::Arc;
use phper::sys::zval;
use phper::values::ExecuteData;

pub struct Psr18Plugin {
    handlers: Vec<Arc<dyn Handler + Send + Sync>>,
}

impl Psr18Plugin {
    pub fn new() -> Self {
        Self {
            handlers: vec![
                Arc::new(Psr18Handler),
            ],
        }
    }
}

//use Psr\Http\Client\ClientInterface;
//use Psr\Http\Message\RequestInterface;
//use Psr\Http\Message\ResponseInterface;

impl Plugin for Psr18Plugin {
    fn is_enabled(&self) -> bool {
        true
    }
    fn get_handlers(&self) -> Vec<Arc<dyn Handler + Send + Sync>> {
        self.handlers.clone()
    }
}

pub struct Psr18Handler;

impl Handler for Psr18Handler {
    fn get_functions(&self) -> Vec<String> {
        vec![]
    }
    fn get_interfaces(&self) -> Vec<String> {
        vec![
            r"Psr\Http\Client\ClientInterface::sendRequest".to_string(),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Self::pre_callback),
            post_observe: Some(Self::post_callback),
        }
    }
}

impl Psr18Handler {
    unsafe extern "C" fn pre_callback(_exec_data: *mut ExecuteData, _span_details: &mut SpanDetails) {
        println!("Psr18Handler::pre_callback");
        
    }

    unsafe extern "C" fn post_callback(
        _exec_data: *mut ExecuteData,
        _span_ref: &SpanRef,
        _retval: *mut phper::sys::zval,
    ) {
        println!("Psr18Handler::post_callback");
    }
}
