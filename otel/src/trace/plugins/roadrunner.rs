use phper::sys::{zend_execute_data, zend_observer_fcall_handlers, _zval_struct, zval};
use super::super::plugin::{Handler, Plugin};
pub struct RoadRunnerPlugin;

impl Plugin for RoadRunnerPlugin {
    fn should_handle(&self) -> bool {
        // Check if RoadRunner PSR-7 environment variables exist
        std::env::var("RR_MODE").map_or(false, |v| v == "http")
    }

    fn get_handlers(&self) -> zend_observer_fcall_handlers {
        zend_observer_fcall_handlers{begin: None, end: None}
    }
}

pub struct WaitRequestHandler;
pub struct RespondHandler;

impl Handler for WaitRequestHandler {
    fn pre_observe(&self, _execute_data: &zend_execute_data) {
        tracing::debug!("WaitRequestHandler pre-observed");
        // Start tracing a new span when request starts
        // let tracer = opentelemetry::global::tracer("roadrunner");
        // let span = tracer.start("http.request");

        // Store span in thread-local storage or request context
        //store_span(span);
    }
    extern "C" fn pre_observe_trampoline(execute_data: *mut zend_execute_data) {
        println!("WaitRequestHandler begin");
    }
    extern "C" fn post_observe_trampoline(execute_data: *mut zend_execute_data, retval: *mut zval) {
        println!("WaitRequestHandler end");
    }
}

impl Handler for RespondHandler {
    fn post_observe(&self, _execute_data: &zend_execute_data, retval: *mut _zval_struct) {

        // Retrieve and end the span after responding
        // if let Some(span) = retrieve_span() {
        //     span.end();
        // }
    }
    extern "C" fn pre_observe_trampoline(execute_data: *mut zend_execute_data) {
        println!("RespondHandler begin");
    }
    extern "C" fn post_observe_trampoline(execute_data: *mut zend_execute_data, retval: *mut zval) {
        println!("RespondHandler end");
    }
}
