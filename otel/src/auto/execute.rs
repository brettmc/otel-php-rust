// Handle auto-instrumentation via zend_execute_ex wrapping (PHP 7)
use phper::{
    sys,
    values::{ExecuteData,ZVal},
};
use std::ptr::null_mut;
use crate::{
    auto::{
        execute_data::{
            get_global_exception,
            get_function_and_class_name,
        },
        plugin_manager::PluginManager,
    }
};
use std::{
    cell::RefCell,
    sync::OnceLock,
};
use dashmap::DashMap;

static PLUGIN_MANAGER: OnceLock<PluginManager> = OnceLock::new();
thread_local! {
    static OBSERVER_MAP: RefCell<DashMap<String, bool>> = RefCell::new(DashMap::new());
}

static mut UPSTREAM_EXECUTE_EX: Option<
    unsafe extern "C" fn(execute_data: *mut sys::zend_execute_data),
> = None;

pub fn init(plugin_manager: PluginManager) {
    tracing::debug!("Execute::init");
    PLUGIN_MANAGER.get_or_init(|| plugin_manager);
    unsafe {
        UPSTREAM_EXECUTE_EX = sys::zend_execute_ex;
        sys::zend_execute_ex = Some(execute_ex);
    }
    tracing::debug!("swapped zend_execute_ex with custom execute_ex");
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

    if let Some(observed) = OBSERVER_MAP.with(|map| map.borrow().get(&key).map(|r| *r)) {
        if !observed {
            // We already know we're not interested in this function
            tracing::trace!("execute_ex: {} already seen and skipped", key);
            upstream_execute_ex(Some(exec_data));
            return;
        }
    }

    let plugin_manager = PLUGIN_MANAGER.get().expect("PluginManager not initialized");
    let observer = plugin_manager.get_function_observer(exec_data);
    OBSERVER_MAP.with(|map| {
        map.borrow().insert(key.clone(), observer.is_some()); //observer was found
    });

    //run pre hooks
    if let Some(ref obs) = observer {
        tracing::trace!("execute_ex: Observing: {}", key);
        for hook in obs.pre_hooks() {
            hook(exec_data);
        }
    }

    //run the observed function
    upstream_execute_ex(Some(exec_data));

    //run post hooks
    if let Some(ref obs) = observer {
        let retval_ptr: *mut sys::zval = unsafe { (*execute_data).return_value };
        let retval = if retval_ptr.is_null() {
            &mut ZVal::from(())
        } else {
            unsafe { ZVal::from_mut_ptr(retval_ptr) }
        };
        for hook in obs.post_hooks() {
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