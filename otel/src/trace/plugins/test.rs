use crate::trace::plugin::{Handler, HandlerCallbacks, Plugin};
use std::sync::Arc;

pub struct TestPlugin {
    handlers: Vec<Arc<dyn Handler + Send + Sync>>,
}

impl TestPlugin {
    pub fn new() -> Self {
        Self {
            handlers: vec![Arc::new(DemoHandler)],
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
}

pub struct DemoHandler;

impl Handler for DemoHandler {
    fn matches(&self, fqn: &str) -> bool {
        let known_functions = &[
            "DemoClass::test",
            "DemoClass::inner",
            "demoFunction",
            "phpversion",
        ];
        if known_functions.iter().any(|&name| fqn == name) {
            //println!("DemoHandler::matched: {}", fqn);
            return true;
        }
        //println!("DemoHandler::not matched: {}", fqn);
        false
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Self::pre_callback),
            post_observe: Some(Self::post_callback),
        }
    }
}

impl DemoHandler {
    unsafe extern "C" fn pre_callback(_execute_data: *mut phper::sys::zend_execute_data) {
        //println!("DemoHandler::pre_callback");
    }

    unsafe extern "C" fn post_callback(
        _execute_data: *mut phper::sys::zend_execute_data,
        _retval: *mut phper::sys::zval,
    ) {
        //println!("DemoHandler::post_callback");
    }
}
