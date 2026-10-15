// Handle auto-instrumentation via zend_execute_ex wrapping (PHP 7)
use phper::{
    sys,
    values::{ExecuteData,ZVal},
};
use crate::{
    auto::{
        execute_data::{
            get_global_exception,
        },
        guard,
        plugin::FunctionObserver,
        plugin_manager::get_global as get_plugin_manager,
    }
};
use std::{
    ptr::null_mut,
};

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
    G: Fn(&FunctionObserver, &mut ExecuteData, &mut ZVal),
{
    let exec_data = match exec_data {
        Some(data) => data,
        None => {
            upstream(None, return_value);
            return;
        }
    };

    // Resolving the observer and running the pre hooks are guarded together: this runs
    // inside an `extern "C"` call from the Zend VM, where unwinding is undefined behaviour
    // and there is no PHP frame to throw into. Whatever happens here, the call below must
    // still reach the engine, so instrumentation failures degrade to a plain call.
    let observer = guard("execute pre hooks", || {
        let Some(plugin_manager) = get_plugin_manager() else {
            tracing::error!("Execute::PluginManager not initialized, skipping instrumentation");
            return None;
        };
        // A poisoned lock means a previous reader panicked; the manager is only mutated
        // during MINIT, so its contents are still sound to read.
        let plugin_manager = match plugin_manager.read() {
            Ok(manager) => manager,
            Err(poisoned) => poisoned.into_inner(),
        };
        let observer = plugin_manager.get_function_observer(exec_data);

        if let Some(ref obs) = observer {
            for hook in obs.pre_hooks() {
                hook(exec_data);
            }
        }
        observer
    })
    .flatten();

    // Not every call has a return value slot, but the post hooks take a `&mut ZVal`.
    // The fallback is declared here (rather than inside the `else`) so that it outlives
    // the borrow below without having to leak it.
    let mut fallback = ZVal::from(());
    let retval: &mut ZVal = match return_value {
        Some(rv) => rv,
        None => &mut fallback,
    };

    upstream(Some(exec_data), Some(retval));

    // Pass the observer we already resolved, rather than looking it up a second time.
    if let Some(ref observer) = observer {
        guard("execute post hooks", || {
            run_post_hooks(observer, exec_data, retval);
        });
    }
}

unsafe extern "C" fn execute_ex(execute_data: *mut sys::zend_execute_data) {
    let exec_data = unsafe{ExecuteData::try_from_mut_ptr(execute_data)};
    handle_execution(
        exec_data,
        None,
        |ed, _| upstream_execute_ex(ed),
        |observer, exec_data, fallback_retval| {
            //get return value via a pointer to avoid double-borrowing exec_data
            let retval = unsafe {
                exec_data.get_return_value_mut_ptr()
                    .as_mut()
                    .unwrap_or(fallback_retval)
            };

            for hook in observer.post_hooks() {
                hook(exec_data, retval, get_global_exception());
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
        |observer, exec_data, retval| {
            for hook in observer.post_hooks() {
                hook(exec_data, retval, get_global_exception());
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
