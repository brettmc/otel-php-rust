//! Auto-instrumentation by wrapping `zend_execute_ex`/`zend_execute_internal` (PHP 7.x).
//!
//! This is one of two mutually exclusive backends, selected by `build.rs`: this module is
//! compiled under `otel_observer_not_supported`, and `auto::observer` takes its place on
//! PHP 8+ under `otel_observer_supported`, where the observer API makes this unnecessary.
//! Because PHP 8 containers never build this file, a normal `cargo check` does not
//! type-check it. To check it:
//!
//! ```sh
//! RUSTFLAGS="--cfg otel_observer_not_supported" cargo check --features test
//! ```
//!
//! [`init`] saves the engine's own handlers and installs ours in their place, so every call
//! to a userland function (`execute_ex`) or an internal one (`execute_internal`) passes
//! through here — unlike the observer API, there is no per-function opt-in, so the hot path
//! is every call in the process and hook lookup happens inline.
//!
//! Both entry points funnel into [`handle_execution`], whose one hard requirement is that
//! the upstream handler is always reached: these are `extern "C"` frames called by the Zend
//! VM, so unwinding is undefined behaviour and swallowing the call would stop the program
//! executing. Instrumentation work is therefore wrapped in [`crate::auto::guard`] and a
//! failure degrades to a plain, uninstrumented call.
use crate::auto::{
    execute_data::get_global_exception, guard, plugin::FunctionObserver,
    plugin_manager::get_global as get_plugin_manager,
};
use phper::{
    sys,
    values::{ExecuteData, ZVal},
};
use std::ptr::null_mut;

static mut UPSTREAM_EXECUTE_EX: Option<
    unsafe extern "C" fn(execute_data: *mut sys::zend_execute_data),
> = None;
static mut UPSTREAM_EXECUTE_INTERNAL: Option<
    unsafe extern "C" fn(execute_data: *mut sys::zend_execute_data, return_value: *mut sys::zval),
> = None;

pub fn init() {
    tracing::debug!("Execute::init");
    unsafe {
        UPSTREAM_EXECUTE_EX = sys::zend_execute_ex;
        sys::zend_execute_ex = Some(execute_ex);

        UPSTREAM_EXECUTE_INTERNAL = sys::zend_execute_internal;
        sys::zend_execute_internal = Some(execute_internal);
    }
    tracing::debug!(
        "swapped zend_execute_ex and zend_execute_internal with custom implementations"
    );
}

/// Handle execution data and return value, invoking upstream functions and post hooks. This function
/// is used by the custom `execute_ex` and `execute_internal` functions to manage the execution flow
/// and apply any registered hooks from the plugin manager.
fn handle_execution<F, G>(
    exec_data: Option<&mut ExecuteData>,
    return_value: Option<&mut ZVal>,
    upstream: F,
    run_post_hooks: G,
) where
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
    let exec_data = unsafe { ExecuteData::try_from_mut_ptr(execute_data) };
    handle_execution(
        exec_data,
        None,
        |ed, _| upstream_execute_ex(ed),
        |observer, exec_data, fallback_retval| {
            //get return value via a pointer to avoid double-borrowing exec_data
            let retval = unsafe {
                exec_data
                    .get_return_value_mut_ptr()
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
    let exec_data = unsafe { ExecuteData::try_from_mut_ptr(execute_data) };
    let ret_val = unsafe { ZVal::try_from_mut_ptr(return_value) };

    handle_execution(
        exec_data,
        ret_val,
        upstream_execute_internal,
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
fn upstream_execute_internal(
    execute_data: Option<&mut ExecuteData>,
    return_value: Option<&mut ZVal>,
) {
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
