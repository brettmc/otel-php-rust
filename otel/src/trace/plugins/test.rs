use phper::sys::{zend_execute_data, zval, zend_observer_fcall_handlers, _zval_struct};
use super::super::plugin::{Handler, Plugin};
use phper::values::{
    ExecuteData,
};
use std::sync::Arc;
use crate::trace::plugin::HandlerCallbacks;

pub struct TestPlugin;

impl Plugin for TestPlugin {
    fn should_handle(&self) -> bool {
        true
    }

    fn get_handlers(&self) -> Vec<Arc<dyn Handler>> {
        vec![Arc::new(TestHandler)]
    }
}

pub struct TestHandler;

impl Handler for TestHandler {
    fn matches(&self, _execute_data: &ExecuteData) -> bool {
        tracing::debug!("TestHandler::matches");
        true
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Self::pre_observe_trampoline),
            post_observe: Some(Self::post_observe_trampoline),
        }
    }
}

impl TestHandler {
    extern "C" fn pre_observe_trampoline(execute_data: *mut phper::sys::zend_execute_data) {
        println!("TestHandler: Function Begin!");
    }

    extern "C" fn post_observe_trampoline(execute_data: *mut phper::sys::zend_execute_data, retval: *mut phper::sys::zval) {
        println!("TestHandler: Function End!");
    }
}
