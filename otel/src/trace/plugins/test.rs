use phper::sys::{zend_execute_data, zval, zend_observer_fcall_handlers, _zval_struct};
use super::super::plugin::{Handler, Plugin};
pub struct TestPlugin;

impl Plugin for TestPlugin {
    fn should_handle(&self) -> bool {
        true
    }

    fn get_handlers(&self) -> zend_observer_fcall_handlers {
        zend_observer_fcall_handlers{begin: None, end: None}
    }
}

pub struct TestHandler;

impl Handler for TestHandler {
    fn pre_observe(&self, _execute_data: &zend_execute_data) {
        tracing::debug!("default pre-observed");
        // Start tracing a new span when request starts
        // let tracer = opentelemetry::global::tracer("roadrunner");
        // let span = tracer.start("http.request");

        // Store span in thread-local storage or request context
        //store_span(span);
    }
    fn post_observe(&self, _execute_data: &zend_execute_data, _retval: *mut _zval_struct) {
        tracing::debug!("default post-observed");
        // Retrieve and end the span after responding
        // if let Some(span) = retrieve_span() {
        //     span.end();
        // }
    }

    extern "C" fn pre_observe_trampoline(execute_data: *mut zend_execute_data) {
        println!("TestHandler begin");
    }
    extern "C" fn post_observe_trampoline(execute_data: *mut zend_execute_data, retval: *mut zval) {
        println!("TestHandler end");
    }
}
