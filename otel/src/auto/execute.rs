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
    collections::HashMap,
    sync::OnceLock,
};

static PLUGIN_MANAGER: OnceLock<PluginManager> = OnceLock::new();
thread_local! {
    static OBSERVER_MAP: std::cell::RefCell<HashMap<String, bool>> = std::cell::RefCell::new(HashMap::new());
}

static mut UPSTREAM_EXECUTE_EX: Option<
    unsafe extern "C" fn(execute_data: *mut sys::zend_execute_data),
> = None;
static mut UPSTREAM_EXECUTE_INTERNAL: Option<
    unsafe extern "C" fn(execute_data: *mut sys::zend_execute_data, return_value: *mut sys::zval)
> = None;

pub fn init(plugin_manager: PluginManager) {
    tracing::debug!("Execute::init");
    PLUGIN_MANAGER.get_or_init(|| plugin_manager);
    unsafe {
        UPSTREAM_EXECUTE_EX = sys::zend_execute_ex;
        sys::zend_execute_ex = Some(execute_ex);

        UPSTREAM_EXECUTE_INTERNAL = sys::zend_execute_internal;
        sys::zend_execute_internal = Some(execute_internal);
    }
    tracing::debug!("swapped zend_execute_ex and zend_execute_internal with custom implementations");
}

// This is our exec function that wraps the upstream PHP one.
// This allows us to execute plugins before&after.
unsafe extern "C" fn execute_ex(execute_data: *mut sys::zend_execute_data) {
    let exec_data = match unsafe{ExecuteData::try_from_mut_ptr(execute_data)} {
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

    if let Some(observed) = OBSERVER_MAP.with(|map| map.borrow_mut().get(&key).copied()) {
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
        map.borrow_mut().insert(key.clone(), observer.is_some()); //store whether the observer was found
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
        // WARNING: if the return value is not used by the calling code (eg assigned to a variable),
        // it may be optimized away and not available in the post hooks :(
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

unsafe extern "C" fn execute_internal(
    execute_data: *mut sys::zend_execute_data,
    return_value: *mut sys::zval,
) {
    tracing::trace!("execute_internal called");
    let (exec_data, return_value) = match (
        unsafe{ExecuteData::try_from_mut_ptr(execute_data)},
        unsafe{ZVal::try_from_mut_ptr(return_value)},
    ) {
        (Some(exec_data), Some(return_value)) => (exec_data, return_value),
        (exec_data, return_value) => {
            tracing::debug!("execute_internal: execute_data or return_value is null, calling upstream directly");
            upstream_execute_internal(exec_data, return_value);
            return;
        }
    };

    let key =
        match get_function_and_class_name(exec_data) {
            Ok((Some(func), Some(cls))) => format!("{}::{}", cls, func),
            Ok((Some(func), None)) => func,
            _ => {
                tracing::debug!("execute_internal: no function or class name found, calling upstream directly");
                upstream_execute_internal(Some(exec_data), Some(return_value));
                return;
            },
        };
    tracing::trace!("execute_internal: key = {}", key);
    if let Some(observed) = OBSERVER_MAP.with(|map| map.borrow_mut().get(&key).copied()) {
        if !observed {
            // We already know we're not interested in this function
            tracing::trace!("execute_internal: {} already seen and skipped", key);
            upstream_execute_internal(Some(exec_data), Some(return_value));
            return;
        }
    }

    let plugin_manager = PLUGIN_MANAGER.get().expect("PluginManager not initialized");
    let observer = plugin_manager.get_function_observer(exec_data);
    OBSERVER_MAP.with(|map| {
        map.borrow_mut().insert(key.clone(), observer.is_some()); //store whether the observer was found
    });

    if let Some(ref obs) = observer {
        tracing::trace!("execute_internal: Observing: {}", key);
        for hook in obs.pre_hooks() {
            hook(exec_data);
        }
    }

    upstream_execute_internal(Some(exec_data), Some(return_value));

    //run post hooks
    if let Some(ref obs) = observer {
        // WARNING: if the return value is not used by the calling code (eg assigned to a variable),
        // it may be optimized away and not available in the post hooks :(
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

/*#[inline]
fn upstream_execute_internal(
    execute_data: *mut sys::zend_execute_data,
    return_value: *mut sys::zval,
) {
    tracing::trace!("upstream_execute_internal called");
    unsafe {
        if let Some(f) = UPSTREAM_EXECUTE_INTERNAL {
            f(execute_data, return_value)
        }
    }
}*/
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
