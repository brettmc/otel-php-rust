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
        guard,
        execute_data::{
            get_fqn,
            get_global_exception,
        },
        plugin_manager::{
            get_cached_observer,
            get_global as get_plugin_manager,
        },
    },
};

pub fn init() {
    tracing::debug!("Observer::init");
    unsafe {
        sys::zend_observer_fcall_register(Some(observer_instrument));
    }
    tracing::debug!("registered fcall handlers");
}

/// # Safety
///
/// Called by the Zend engine as an observer registration handler. `execute_data`
/// must be a valid pointer to the frame being compiled, or null.
pub unsafe extern "C" fn observer_instrument(execute_data: *mut sys::zend_execute_data) -> sys::zend_observer_fcall_handlers {
    const UNOBSERVED: sys::zend_observer_fcall_handlers = sys::zend_observer_fcall_handlers {
        begin: None,
        end: None,
    };

    let observed = guard("observer_instrument", || {
        let Some(exec_data) = (unsafe { ExecuteData::try_from_mut_ptr(execute_data) }) else {
            return false;
        };
        let Some(fqn) = get_fqn(exec_data) else {
            return false;
        };
        tracing::trace!("observer::observer_instrument checking: {}", &fqn);

        let Some(plugin_manager) = get_plugin_manager() else {
            tracing::error!("Observer::PluginManager not initialized, skipping instrumentation");
            return false;
        };
        // A poisoned lock means a previous reader panicked; the manager is only mutated
        // during MINIT, so its contents are still sound to read.
        let plugin_manager = match plugin_manager.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        // Populates the shared cache that the begin/end handlers below read from.
        plugin_manager.get_function_observer(exec_data).is_some()
    });

    if observed != Some(true) {
        return UNOBSERVED;
    }

    sys::zend_observer_fcall_handlers {
        begin: Some(pre_observe_c_function),
        end: Some(post_observe_c_function),
    }
}

/// # Safety
///
/// Called by the Zend engine on function entry. `execute_data` must be a valid
/// pointer to the frame being entered, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pre_observe_c_function(execute_data: *mut sys::zend_execute_data) {
    guard("pre_observe", || {
        let Some(exec_data) = (unsafe { ExecuteData::try_from_mut_ptr(execute_data) }) else {
            return;
        };
        let Some(fqn) = get_fqn(exec_data) else {
            return;
        };
        // Only functions with hooks are ever cached, so no `has_hooks()` re-check is needed.
        let Some(observer) = get_cached_observer(&fqn) else {
            return;
        };
        for hook in observer.pre_hooks() {
            tracing::trace!("running pre hook: {}", fqn);
            hook(exec_data);
        }
    });
}

/// # Safety
///
/// Called by the Zend engine on function exit. `execute_data` must be a valid
/// pointer to the frame being left, or null; `retval` must point to the return
/// value, or be null if the function returned nothing.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn post_observe_c_function(execute_data: *mut sys::zend_execute_data, retval: *mut sys::zval) {
    guard("post_observe", || {
        let Some(exec_data) = (unsafe { ExecuteData::try_from_mut_ptr(execute_data) }) else {
            return;
        };
        let Some(fqn) = get_fqn(exec_data) else {
            return;
        };
        let Some(observer) = get_cached_observer(&fqn) else {
            return;
        };

        // Not every observed call has a return value slot, but the hooks take a `&mut ZVal`.
        // The fallback is bound here so that it outlives the borrow handed to the hooks.
        let mut fallback = ZVal::from(());
        let retval: &mut ZVal = match unsafe { (retval as *mut ZVal).as_mut() } {
            Some(retval) => retval,
            None => &mut fallback,
        };

        for hook in observer.post_hooks() {
            tracing::trace!("running post hook: {}", fqn);
            hook(exec_data, retval, get_global_exception());
        }
    });
}
