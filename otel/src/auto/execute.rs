// Handle auto-instrumentation via zend_execute_ex wrapping (PHP 7)
use phper::{
    sys,
    values::{ExecuteData,ZVal},
};
use std::ptr::null_mut;
use crate::{
    logging,
    auto::{
        execute_data::{
            get_global_exception,
            get_function_and_class_name,
        },
        plugin_manager::PluginManager,
    }
};
use std::sync::{Arc, OnceLock};
use dashmap::DashMap;

static PLUGIN_MANAGER: OnceLock<PluginManager> = OnceLock::new();
static OBSERVER_MAP: OnceLock<Arc<DashMap<String, bool>>> = OnceLock::new();

static mut UPSTREAM_EXECUTE_EX: Option<
    unsafe extern "C" fn(execute_data: *mut sys::zend_execute_data),
> = None;

pub fn init(plugin_manager: PluginManager) {
    tracing::debug!("Execute::init");
    PLUGIN_MANAGER.get_or_init(|| plugin_manager);
    OBSERVER_MAP.get_or_init(|| Arc::new(DashMap::new()));
    unsafe {
        UPSTREAM_EXECUTE_EX = sys::zend_execute_ex;
        sys::zend_execute_ex = Some(execute_ex);
    }
}

// This is our exec function that wraps the upstream PHP one.
// This allows us to execute plugins before&after.
unsafe extern "C" fn execute_ex(execute_data: *mut sys::zend_execute_data) {
    let exec_data = match ExecuteData::try_from_mut_ptr(execute_data) {
        Some(execute_data) => execute_data,
        None => {
            upstream_execute_ex(None);
            return;
        }
    };
    let key =
        match get_function_and_class_name(exec_data) {
            Ok((Some(func), Some(cls))) => format!("{}::{}", cls, func),
            Ok((Some(func), None)) => func,
            _ => {
                upstream_execute_ex(Some(exec_data));
                return;
            },
        };
    let plugin_manager = PLUGIN_MANAGER.get().expect("PluginManager not initialized");
    if let Some(observed) = OBSERVER_MAP.get().and_then(|map| map.get(&key)) {
        if !*observed {
            // We already know we're not interested in this function
            tracing::trace!("execute_ex: {} already seen and skipped", key);
            upstream_execute_ex(Some(exec_data));
            return;
        } else {
            tracing::trace!("execute_ex: {} already seen and observed", key);
        }
    }

    if let Some(observer) = plugin_manager.get_function_observer(exec_data) {
        tracing::trace!("execute_ex: Observing: {}", key);
        OBSERVER_MAP.get().unwrap().insert(key, true);
        // Run pre hooks
        for hook in observer.pre_hooks() {
            hook(exec_data);
        }
    } else {
        OBSERVER_MAP.get().unwrap().insert(key, false);
    }

    upstream_execute_ex(Some(exec_data));

    if let Some(observer) = plugin_manager.get_function_observer(exec_data) {
        let retval_ptr: *mut sys::zval = unsafe { (*execute_data).return_value };
        let retval = if retval_ptr.is_null() {
            &mut ZVal::from(())
        } else {
            unsafe { ZVal::from_mut_ptr(retval_ptr) }
        };
        // Run post hooks
        for hook in observer.post_hooks() {
            hook(exec_data, retval, get_global_exception());
        }
    }
}

#[inline]
fn upstream_execute_ex(execute_data: Option<&mut ExecuteData>) {
    unsafe {
        if let Some(f) = UPSTREAM_EXECUTE_EX {
            f(execute_data
                .map(ExecuteData::as_mut_ptr)
                .unwrap_or(null_mut()))
        }
    }
}