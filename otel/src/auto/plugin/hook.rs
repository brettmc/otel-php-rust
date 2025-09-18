use crate::{
    auto::{
        execute_data::{
            get_function_and_class_name,
            get_function_arguments,
            get_file_and_line,
            get_this_or_called_scope,
            set_parameter_slots, // updated import
        },
        plugin::{Handler, HandlerList, HandlerSlice, HandlerCallbacks, Plugin},
        utils::should_trace,
    },
};
use std::sync::Arc;
use std::collections::HashMap;
use phper::{
    values::{ExecuteData, ZVal},
    objects::ZObj,
    arrays::{ZArray},
};
use phper::alloc::{RefClone, ToRefOwned};

// Thread-local registry for hooks
thread_local! {
    static HOOK_REGISTRY: std::cell::RefCell<HashMap<(Option<String>, String), (Vec<ZVal>, Vec<ZVal>)>> = std::cell::RefCell::new(HashMap::new());
}

pub fn add_hook(
    class: Option<String>,
    function: String,
    pre: Option<ZVal>,
    post: Option<ZVal>,
) {
    let key = (class, function);
    HOOK_REGISTRY.with(|registry| {
        let mut reg = registry.borrow_mut();
        reg.entry(key)
            .and_modify(|(pre_hooks, post_hooks)| {
                if let Some(pre_hook) = pre.clone() {
                    pre_hooks.push(pre_hook);
                }
                if let Some(post_hook) = post.clone() {
                    post_hooks.insert(0, post_hook);
                }
            })
            .or_insert_with(|| {
                (
                    pre.clone().map_or_else(Vec::new, |h| vec![h]),
                    post.clone().map_or_else(Vec::new, |h| vec![h]),
                )
            });
    });
}

/// A plugin that manages hooks for function execution, allowing pre- and post-execution callbacks.
/// This replicates opentelemetry-php-instrumentation's hook functionality.
pub struct HookPlugin {
    handlers: HandlerList,
}

impl HookPlugin {
    pub fn new() -> Self {
        Self {
            handlers: vec![
                Arc::new(HookHandler),
            ],
        }
    }
}

impl Plugin for HookPlugin {
    fn get_handlers(&self) -> &HandlerSlice {
        &self.handlers
    }
    fn get_name(&self) -> &str {
        "hook"
    }
    fn request_shutdown(&self) {
        tracing::debug!("Plugin::request_shutdown: {}", self.get_name());
        HOOK_REGISTRY.with(|registry| {
            registry.borrow_mut().clear();
        });
    }
}

pub struct HookHandler;

impl Handler for HookHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        HOOK_REGISTRY.with(|registry| {
            registry.borrow().keys().map(|(class, function)| {
                (class.clone(), function.clone())
            }).collect()
        })
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Box::new(|exec_data| unsafe {
                Self::pre_callback(exec_data)
            })),
            post_observe: Some(Box::new(|exec_data, retval, exception| unsafe {
                Self::post_callback(exec_data, retval, exception)
            })),
        }
    }
}

impl HookHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        tracing::trace!("HookHandler::pre_callback");
        let exec_data_ref = unsafe { &mut *exec_data };
        let (file, line) = get_file_and_line(exec_data_ref).unwrap_or_default();
        if let Some((pre_hooks, _)) = find_hook_for_exec_data(exec_data_ref) {
            let (function, class) = get_function_and_class_name(exec_data_ref).unwrap_or_default();
            tracing::trace!("HookHandler::pre_callback: found hooks for {:?}::{:?}", class, function);
            let obj_zval = get_this_or_called_scope(exec_data_ref);
            let declaring_scope_zval = ZVal::from(class.clone());
            let function_zval = ZVal::from(function.clone().unwrap_or_default());
            let filename_zval = ZVal::from(file.clone());
            let lineno_zval = ZVal::from(line as i64);
            let withspan_zval = ZVal::from(ZArray::new());
            let attributes = ZVal::from(ZArray::new());

            for mut pre_hook in pre_hooks.clone() {
                let arguments = get_function_arguments(exec_data_ref); //arguments can mutate
                // Debug print all values before calling the hook
                tracing::trace!(
                    "PreHook values: obj_zval={:?}, arguments={:?}, class_zval={:?}, function_zval={:?}, filename_zval={:?}, lineno_zval={:?}",
                    obj_zval, arguments, declaring_scope_zval, function_zval, filename_zval, lineno_zval
                );
                if let Some(zobj) = pre_hook.as_mut_z_obj() {
                    //object, params, class, function, filename, lineno
                    if let Ok(replaced) = zobj.call("__invoke", [
                        obj_zval.clone(),
                        arguments.clone(),
                        declaring_scope_zval.clone(),
                        function_zval.clone(),
                        filename_zval.clone(),
                        lineno_zval.clone(),
                        withspan_zval.clone(),
                        attributes.clone(),
                    ]) {
                        if let Some(arr) = replaced.as_z_arr() {
                            // Use set_parameter_slots to apply all replacements at once
                            set_parameter_slots(exec_data_ref, arr.iter().map(|(k, v)| (k, v.clone())));
                        }
                    }
                }
            }
            tracing::trace!("HookHandler::post_callback: calling handle_missing_default_args");
            crate::auto::execute_data::handle_missing_default_args(exec_data_ref);
        } else {
            let (function, class) = get_function_and_class_name(exec_data_ref).unwrap_or_default();
            tracing::trace!("HookHandler::pre_callback: no pre-hooks found for {:?}::{:?}", class, function);
        }
    }

    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        retval: &mut ZVal,
        exception: Option<&mut ZObj>
    ) {
        tracing::trace!("HookHandler::post_callback");
        let exec_data_ref = unsafe { &mut *exec_data };
        let (file, line) = get_file_and_line(exec_data_ref).unwrap_or_default();
        if let Some((_, post_hooks)) = find_hook_for_exec_data(exec_data_ref) {
            let (function, class) = get_function_and_class_name(exec_data_ref).unwrap_or_default();
            let obj_zval = get_this_or_called_scope(exec_data_ref);
            let arguments = get_function_arguments(exec_data_ref);
            let exception_zval = match exception {
                Some(zobj) => ZVal::from(zobj.to_ref_owned().ref_clone()),
                None => ZVal::from(()),
            };
            let declaring_scope_zval = ZVal::from(class.clone());
            let function_zval = ZVal::from(function.clone().unwrap_or_default());
            let filename_zval = ZVal::from(file.clone());
            let lineno_zval = ZVal::from(line as i64);

            // Print all parameters for verification
            tracing::debug!("arguments: {:?}", arguments);
            let idx = 1;
            let zv_verify = exec_data_ref.get_parameter(idx);
            tracing::debug!("PostHook: verified parameter at index {} is now value={:?}", idx, zv_verify);

            for mut post_hook in post_hooks.clone() {
                // Debug print all values before calling the hook
                tracing::debug!(
                    "PostHook values: obj_zval={:?}, arguments={:?}, retval={:?}, exception={:?}",
                    obj_zval, arguments, retval, exception_zval
                );
                if let Some(zobj) = post_hook.as_mut_z_obj() {
                    //object, params, ?returnval, ?exception, declaring scope, function name, filename, line number
                    if let Ok(modified_return_value) = zobj.call("__invoke", [
                        obj_zval.clone(),
                        arguments.clone(),
                        retval.clone(),
                        exception_zval.clone(),
                        declaring_scope_zval.clone(),
                        function_zval.clone(),
                        filename_zval.clone(),
                        lineno_zval.clone(),
                    ]) {
                        *retval = modified_return_value;
                    }
                }
            }
        } else {
            tracing::warn!("HookHandler::post_callback: no post-hooks found for this function");
        }
    }
}

fn find_hook_for_exec_data(
    exec_data: &ExecuteData,
) -> Option<(Vec<ZVal>, Vec<ZVal>)> {
    HOOK_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        // Build targets from registry keys
        let targets: Vec<(Option<String>, String)> = registry.keys().cloned().collect();
        // Use should_trace to determine if this exec_data matches any hook
        let func = exec_data.func();
        if should_trace(func, &targets) {
            // Find the first matching hook using should_trace logic
            for (key, hooks) in registry.iter() {
                if should_trace(func, &[key.clone()]) {
                    return Some(hooks.clone());
                }
            }
        }
        None
    })
}