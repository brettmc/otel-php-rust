// Handle auto-instrumentation via the observer API (PHP 8+)
use phper::{
    sys,
    values::{
        ExecuteData,
        ZVal,
    }
};
use crate::{
    auto::{
        execute_data::{
            get_fqn,
            get_global_exception,
        },
        plugin::{
            FunctionObserver,
        },
        plugin_manager::{
            get_global as get_plugin_manager,
        },
    },
};
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

static FUNCTION_OBSERVERS: OnceLock<RwLock<HashMap<String, Arc<FunctionObserver>>>> = OnceLock::new();

pub fn init() {
    tracing::debug!("Observer::init");
    FUNCTION_OBSERVERS.get_or_init(|| RwLock::new(HashMap::new()));
    unsafe {
        sys::zend_observer_fcall_register(Some(observer_instrument));
    }
    tracing::debug!("registered fcall handlers");
}

pub unsafe extern "C" fn observer_instrument(execute_data: *mut sys::zend_execute_data) -> sys::zend_observer_fcall_handlers {
    if let Some(exec_data) = unsafe{ExecuteData::try_from_mut_ptr(execute_data)} {
        if let Some(fqn) = get_fqn(exec_data) {
            tracing::trace!("observer::observer_instrument checking: {}", &fqn);
            let plugin_manager = get_plugin_manager()
                .expect("PluginManager not initialized")
                .read()
                .unwrap();
            if let Some(observer) = plugin_manager.get_function_observer(exec_data) {
                let observers = FUNCTION_OBSERVERS.get().expect("Function observer not initialized");
                let mut lock = observers.write().unwrap();
                lock.insert(fqn, observer);

                static mut HANDLERS: sys::zend_observer_fcall_handlers = sys::zend_observer_fcall_handlers {
                    begin: Some(pre_observe_c_function),
                    end: Some(post_observe_c_function),
                };

                return unsafe { HANDLERS };
            }
        }
    }

    sys::zend_observer_fcall_handlers {
        begin: None,
        end: None,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pre_observe_c_function(execute_data: *mut sys::zend_execute_data) {
    if let Some(exec_data) = unsafe{ExecuteData::try_from_mut_ptr(execute_data)} {
        if let Some(fqn) = get_fqn(exec_data) {
            let observers = FUNCTION_OBSERVERS.get().expect("Function observer not initialized");
            let lock = observers.read().unwrap();
            if let Some(observer) = lock.get(&fqn) {
                if observer.has_hooks() {
                    for hook in observer.pre_hooks() {
                        tracing::trace!("running pre hook: {}", fqn);
                        hook(&mut *exec_data);
                    }
                }
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn post_observe_c_function(execute_data: *mut sys::zend_execute_data, retval: *mut sys::zval) {
    if let Some(exec_data) = unsafe{ExecuteData::try_from_mut_ptr(execute_data)} {
        if let Some(fqn) = get_fqn(exec_data) {
            let observers = FUNCTION_OBSERVERS.get().expect("Function observer not initialized");
            let lock = observers.read().unwrap();
            if let Some(observer) = lock.get(&fqn) {
                let retval = if retval.is_null() {
                    &mut ZVal::from(())
                } else {
                    unsafe { (retval as *mut ZVal).as_mut().unwrap() }
                };
                for hook in observer.post_hooks() {
                    tracing::trace!("running post hook: {}", fqn);
                    hook(&mut *exec_data, retval, get_global_exception());
                }
            }
        }
    }
}