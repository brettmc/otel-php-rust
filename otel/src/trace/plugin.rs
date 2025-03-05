use phper::{
    functions::ZFunc,
};
use std::sync::Arc;

pub trait Plugin: Send + Sync {
    /// Determines whether this plugin is enabled. Could be based on .ini config, or custom logic.
    fn is_enabled(&self) -> bool;
    fn get_handlers(&self) -> Vec<Arc<dyn Handler>>;
}

pub trait Handler: Send + Sync {
    /// Should the function in execute data be observed by this plugin?
    fn matches(&self, func: &ZFunc) -> bool;
    fn get_callbacks(&self) -> HandlerCallbacks;
}

pub struct HandlerCallbacks {
    pub pre_observe: Option<unsafe extern "C" fn(*mut phper::sys::zend_execute_data)>,
    pub post_observe: Option<unsafe extern "C" fn(*mut phper::sys::zend_execute_data, *mut phper::sys::zval)>,
}
