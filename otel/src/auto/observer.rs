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
        }
    },
    PluginManager
};
use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

static FUNCTION_OBSERVERS: OnceLock<RwLock<HashMap<String, FunctionObserver>>> = OnceLock::new();
static PLUGIN_MANAGER: OnceLock<PluginManager> = OnceLock::new();

pub fn init(plugin_manager: PluginManager) {
    tracing::debug!("Observer::init");
    PLUGIN_MANAGER.get_or_init(|| plugin_manager);
    FUNCTION_OBSERVERS.get_or_init(|| RwLock::new(HashMap::new()));
    unsafe {
        sys::zend_observer_fcall_register(Some(observer_instrument));
    }
    tracing::debug!("registered fcall handlers");
}

pub unsafe extern "C" fn observer_instrument(execute_data: *mut sys::zend_execute_data) -> sys::zend_observer_fcall_handlers {
    if let Some(exec_data) = ExecuteData::try_from_mut_ptr(execute_data) {
        let fqn = get_fqn(exec_data);
        //tracing::trace!("observer::observer_instrument checking: {}", fqn);
        let plugin_manager = PLUGIN_MANAGER.get().expect("PluginManager not initialized");
        if let Some(observer) = plugin_manager.get_function_observer(exec_data) {
            let observers = FUNCTION_OBSERVERS.get().expect("Function observer not initialized");
            let fqn = fqn.to_string();
            let mut lock = observers.write().unwrap();
            lock.insert(fqn, observer);

            static mut HANDLERS: sys::zend_observer_fcall_handlers = sys::zend_observer_fcall_handlers {
                begin: Some(pre_observe_c_function),
                end: Some(post_observe_c_function),
            };

            return unsafe { HANDLERS };
        }
    }

    sys::zend_observer_fcall_handlers {
        begin: None,
        end: None,
    }
}

#[no_mangle]
pub unsafe extern "C" fn pre_observe_c_function(execute_data: *mut sys::zend_execute_data) {
    if let Some(exec_data) = ExecuteData::try_from_mut_ptr(execute_data) {
        let fqn = get_fqn(exec_data);

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

#[no_mangle]
pub unsafe extern "C" fn post_observe_c_function(execute_data: *mut sys::zend_execute_data, retval: *mut sys::zval) {
    if let Some(exec_data) = ExecuteData::try_from_mut_ptr(execute_data) {
        let fqn = get_fqn(exec_data);

        let observers = FUNCTION_OBSERVERS.get().expect("Function observer not initialized");
        let lock = observers.read().unwrap();
        if let Some(observer) = lock.get(&fqn) {
            //TODO use Option<ZVal> ??
            let retval = if retval.is_null() {
                &mut ZVal::from(())
            } else {
                (retval as *mut ZVal).as_mut().unwrap()
            };

            for hook in observer.post_hooks() {
                tracing::trace!("running post hook: {}", fqn);
                hook(&mut *exec_data, retval, get_global_exception());
            }
        }
    }
}