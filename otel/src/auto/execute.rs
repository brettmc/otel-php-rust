// Handle auto-instrumentation via zend_execute_ex wrapping (PHP 7)
use phper::{sys, values::ExecuteData};
use phper::values::ZVal;
use std::ptr::null_mut;
use crate::auto::plugin_manager::PluginManager;
use crate::auto::execute_data::get_global_exception;
use std::sync::OnceLock;

static PLUGIN_MANAGER: OnceLock<PluginManager> = OnceLock::new();

static mut UPSTREAM_EXECUTE_EX: Option<
    unsafe extern "C" fn(execute_data: *mut sys::zend_execute_data),
> = None;

// This function swaps out the PHP exec function for our own. Allowing us to wrap it.
pub fn register_exec_functions() {
    PLUGIN_MANAGER.get_or_init(|| PluginManager::new());
    unsafe {
        UPSTREAM_EXECUTE_EX = sys::zend_execute_ex;
        sys::zend_execute_ex = Some(execute_ex);
    }
}

// This is our exec function that wraps the upstream PHP one.
// This allows us to gather our execution timing data.
unsafe extern "C" fn execute_ex(execute_data: *mut sys::zend_execute_data) {
    let exec_data = match ExecuteData::try_from_mut_ptr(execute_data) {
        Some(execute_data) => execute_data,
        None => {
            upstream_execute_ex(None);
            return;
        }
    };

    let plugin_manager = PLUGIN_MANAGER.get().expect("PluginManager not initialized");
    if let Some(observer) = plugin_manager.get_function_observer(exec_data) {
        // Run pre hooks
        for hook in observer.pre_hooks() {
            hook(exec_data);
        }
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

    // let class_name = execute_data
    //     .func()
    //     .get_class()
    //     .and_then(|class_entry| class_entry.get_name().to_str().ok());
    //
    // let function_name = execute_data
    //     .func()
    //     .get_function_name()
    //     .and_then(|zstr| zstr.to_str().ok());
    //
    // tracing::trace!("Executing function: {}::{}",
    //     class_name.unwrap_or("<unknown>"),
    //     function_name.unwrap_or("<unknown>"),
    // );
    //
    // // Run the upstream function and record the duration.
    // upstream_execute_ex(Some(execute_data));
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