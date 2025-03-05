use phper::sys::{zval, zend_execute_data, zend_observer_fcall_handlers, _zval_struct};

pub trait Plugin: Send + Sync {
    /// Determines whether this plugin should be applied.
    fn should_handle(&self) -> bool;

    fn get_handlers(&self) -> zend_observer_fcall_handlers;
}

pub trait Handler: Send + Sync {
    /// Called when the observed function is entered.
    fn pre_observe(&self, _execute_data: &zend_execute_data) {}

    /// Called after the observed function returns.
    fn post_observe(&self, _execute_data: &zend_execute_data, retval: *mut _zval_struct) {}

    fn get_callbacks(&self) -> zend_observer_fcall_handlers
    {
        zend_observer_fcall_handlers {
            begin: Some( Self::pre_observe_trampoline),
            end: Some( Self::post_observe_trampoline)
        }
    }

    extern "C" fn pre_observe_trampoline(execute_data: *mut zend_execute_data);
    extern "C" fn post_observe_trampoline(execute_data: *mut zend_execute_data, retval: *mut zval);
}
