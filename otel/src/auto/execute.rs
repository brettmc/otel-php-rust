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
        plugin_manager::{
            get_global as get_plugin_manager,
            PluginManager,
        },
    }
};
use std::{
    collections::HashMap,
};

thread_local! {
    static OBSERVER_MAP: std::cell::RefCell<HashMap<String, bool>> = std::cell::RefCell::new(HashMap::new());
}

static mut UPSTREAM_EXECUTE_EX: Option<
    unsafe extern "C" fn(execute_data: *mut sys::zend_execute_data),
> = None;
static mut UPSTREAM_EXECUTE_INTERNAL: Option<
    unsafe extern "C" fn(execute_data: *mut sys::zend_execute_data, return_value: *mut sys::zval)
> = None;

pub fn init() {
    tracing::debug!("Execute::init");
    unsafe {
        UPSTREAM_EXECUTE_EX = sys::zend_execute_ex;
        sys::zend_execute_ex = Some(execute_ex);

        UPSTREAM_EXECUTE_INTERNAL = sys::zend_execute_internal;
        sys::zend_execute_internal = Some(execute_internal);
    }
    tracing::debug!("swapped zend_execute_ex and zend_execute_internal with custom implementations");
}

/// Handle execution data and return value, invoking upstream functions and post hooks. This function
/// is used by the custom `execute_ex` and `execute_internal` functions to manage the execution flow
/// and apply any registered hooks from the plugin manager.
fn handle_execution<F, G>(
    exec_data: Option<&mut ExecuteData>,
    return_value: Option<&mut ZVal>,
    upstream: F,
    run_post_hooks: G,
)
where
    F: Fn(Option<&mut ExecuteData>, Option<&mut ZVal>),
    G: Fn(&PluginManager, &str, &mut ExecuteData, &mut ZVal),
{
    let exec_data = match exec_data {
        Some(data) => data,
        None => {
            upstream(None, return_value);
            return;
        }
    };

    let key = match get_function_and_class_name(exec_data) {
        Ok((Some(func), Some(cls))) => format!("{}::{}", cls, func),
        Ok((Some(func), None)) => func,
        _ => {
            upstream(Some(exec_data), return_value);
            return;
        },
    };

    if let Some(observed) = OBSERVER_MAP.with(|map| map.borrow_mut().get(&key).copied()) {
        if !observed {
            upstream(Some(exec_data), return_value);
            return;
        }
    }

    let plugin_manager = get_plugin_manager()
        .expect("PluginManager not initialized")
        .read()
        .unwrap();
    let observer = plugin_manager.get_function_observer(exec_data);
    OBSERVER_MAP.with(|map| {
        map.borrow_mut().insert(key.clone(), observer.is_some());
    });

    if let Some(ref obs) = observer {
        for hook in obs.pre_hooks() {
            hook(exec_data);
        }
    }

    // Destructure return_value before moving it
    let retval: &mut ZVal = if let Some(rv) = return_value {
        rv
    } else {
        let fallback = ZVal::from(());
        // Use Box to extend the lifetime so retval can be returned
        Box::leak(Box::new(fallback))
    };

    upstream(Some(exec_data), Some(retval));

    if let Some(ref _observer) = observer {
        run_post_hooks(&plugin_manager, &key, exec_data, retval);
    }
}

unsafe extern "C" fn execute_ex(execute_data: *mut sys::zend_execute_data) {
    let exec_data = unsafe{ExecuteData::try_from_mut_ptr(execute_data)};
    handle_execution(
        exec_data,
        None,
        |ed, _| upstream_execute_ex(ed),
        |plugin_manager, _key, exec_data, _retval| {
            if let Some(observer) = plugin_manager.get_function_observer(exec_data) {
                let retval_ptr: *mut sys::zval = unsafe { (*execute_data).return_value };
                let mut fallback = ZVal::from(());
                let retval: &mut ZVal = if retval_ptr.is_null() {
                    &mut fallback
                } else {
                    unsafe { ZVal::from_mut_ptr(retval_ptr) }
                };
                for hook in observer.post_hooks() {
                    hook(exec_data, retval, get_global_exception());
                }
            }
        },
    );
}

unsafe extern "C" fn execute_internal(
    execute_data: *mut sys::zend_execute_data,
    return_value: *mut sys::zval,
) {
    let exec_data = unsafe{ExecuteData::try_from_mut_ptr(execute_data)};
    let ret_val = unsafe{ZVal::try_from_mut_ptr(return_value)};

    handle_execution(
        exec_data,
        ret_val,
        |ed, rv| upstream_execute_internal(ed, rv),
        |plugin_manager, _key, exec_data, retval| {
            if let Some(observer) = plugin_manager.get_function_observer(exec_data) {
                for hook in observer.post_hooks() {
                    hook(exec_data, retval, get_global_exception());
                }
            }
        },
    );
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

#[inline]
fn upstream_execute_internal(execute_data: Option<&mut ExecuteData>, return_value: Option<&mut ZVal>) {
    let execute_data = execute_data
        .map(ExecuteData::as_mut_ptr)
        .unwrap_or(null_mut());
    let return_value = return_value.map(ZVal::as_mut_ptr).unwrap_or(null_mut());
    unsafe {
        match UPSTREAM_EXECUTE_INTERNAL {
            Some(f) => f(execute_data, return_value),
            None => sys::execute_internal(execute_data, return_value),
        }
    }
}
