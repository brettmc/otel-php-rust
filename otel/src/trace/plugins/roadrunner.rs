use phper::{
    functions::ZFunc,
};
use std::sync::Arc;
use crate::trace::plugin::{Handler, HandlerCallbacks, Plugin};
pub struct RoadRunnerPlugin;

impl Plugin for RoadRunnerPlugin {
    fn is_enabled(&self) -> bool {
        // Check if RoadRunner PSR-7 environment variables exist
        std::env::var("RR_MODE").map_or(false, |v| v == "http")
    }

    fn get_handlers(&self) -> Vec<Arc<dyn Handler>> {
        vec![Arc::new(WaitRequestHandler), Arc::new(RespondHandler)]
    }
}

pub struct WaitRequestHandler;
pub struct RespondHandler;

impl Handler for WaitRequestHandler {
    fn matches(&self, _func: &ZFunc) -> bool {
        false
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Self::pre_observe_trampoline),
            post_observe: Some(Self::post_observe_trampoline),
        }
    }
}

impl WaitRequestHandler {
    unsafe extern "C" fn pre_observe_trampoline(_execute_data: *mut phper::sys::zend_execute_data) {
        println!("roadrunner: pre_observe");
    }

    extern "C" fn post_observe_trampoline(_execute_data: *mut phper::sys::zend_execute_data, _retval: *mut phper::sys::zval) {
        println!("roadrunner: post_observe");
    }
}

impl Handler for RespondHandler {
    fn matches(&self, _func: &ZFunc) -> bool {
        false
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Self::pre_observe_trampoline),
            post_observe: Some(Self::post_observe_trampoline),
        }
    }
}

impl RespondHandler {
    unsafe extern "C" fn pre_observe_trampoline(_execute_data: *mut phper::sys::zend_execute_data) {
        println!("roadrunner: pre_observe");
    }

    extern "C" fn post_observe_trampoline(_execute_data: *mut phper::sys::zend_execute_data, _retval: *mut phper::sys::zval) {
        println!("roadrunner: post_observe");
    }
}
