use phper::sys::{zval, zend_execute_data, zend_observer_fcall_handlers, _zval_struct};
use phper::values::{
    ExecuteData,
};
use std::sync::Arc;

pub trait Plugin: Send + Sync {
    /// Determines whether this plugin should be applied.
    fn should_handle(&self) -> bool;

    fn get_handlers(&self) -> Vec<Arc<dyn Handler>>;
}

pub trait Handler: Send + Sync {
    fn matches(&self, execute_data: &ExecuteData) -> bool;
    fn get_callbacks(&self) -> HandlerCallbacks;
}

pub struct HandlerCallbacks {
    pub pre_observe: Option<unsafe extern "C" fn(*mut phper::sys::zend_execute_data)>,
    pub post_observe: Option<unsafe extern "C" fn(*mut phper::sys::zend_execute_data, *mut phper::sys::zval)>,
}
