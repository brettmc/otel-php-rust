use crate::{
    auto::{
        execute_data::{
            get_function_and_class_name,
            get_function_arguments,
            get_file_and_line,
            get_this_or_called_scope,
        },
        plugin::{Handler, HandlerList, HandlerSlice, HandlerCallbacks, Plugin},
    },
};
use std::sync::Arc;
use std::collections::HashMap;
use phper::{
    values::{ExecuteData, ZVal},
    objects::ZObj,
};
use phper::alloc::{RefClone, ToRefOwned};
use phper::arrays::ZArray;

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
        let exec_data_ref = unsafe { &mut *exec_data };
        let (file, line) = get_file_and_line(exec_data_ref).unwrap_or_default();
        if let Ok((function, class)) = get_function_and_class_name(exec_data_ref) {
            if let Some(function) = function {
                HOOK_REGISTRY.with(|registry| {
                    if let Some((pre_hooks, _)) = registry.borrow().get(&(class.clone(), function.clone())) {
                        let obj_zval = get_this_or_called_scope(exec_data_ref);
                        let arguments = get_function_arguments(exec_data_ref);
                        let declaring_scope_zval = ZVal::from(class.clone());
                        let function_zval = ZVal::from(function.clone());
                        let filename_zval = ZVal::from(file.clone());
                        let lineno_zval = ZVal::from(line as i64);
                        let withspan_zval = ZVal::from(ZArray::new());
                        let attributes = ZVal::from(ZArray::new());

                        for mut pre_hook in pre_hooks.clone() {
                            // Debug print all values before calling the hook
                            tracing::debug!(
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
                                        for (key, value) in arr.iter() {
                                            tracing::debug!("PreHook returned modification: key={:?}, value={:?}", key, value);
                                            let idx_opt = match key {
                                                phper::arrays::IterKey::Index(i) => Some(i as usize),
                                                _ => None, //ignore string keys
                                            };
                                            if let Some(idx) = idx_opt {
                                                exec_data_ref.ensure_parameter_slot(idx);
                                                tracing::debug!("PreHook attempting to modify argument at index {}: {:?}", idx, value);
                                                let zv = exec_data_ref.get_mut_parameter(idx);
                                                tracing::debug!("PreHook: got mutable reference to parameter at index {} value={:?}", idx, zv);
                                                *zv = value.clone();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }
        }
    }

    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        retval: &mut ZVal,
        exception: Option<&mut ZObj>
    ) {
        let exec_data_ref = unsafe { &mut *exec_data };
        let (file, line) = get_file_and_line(exec_data_ref).unwrap_or_default();
        if let Ok((function, class)) = get_function_and_class_name(exec_data_ref) {
            if let Some(function) = function {
                HOOK_REGISTRY.with(|registry| {
                    if let Some((_, post_hooks)) = registry.borrow().get(&(class.clone(), function.clone())) {
                        let obj_zval = get_this_or_called_scope(exec_data_ref);
                        let arguments = get_function_arguments(exec_data_ref);
                        let exception_zval = match exception {
                            Some(zobj) => ZVal::from(zobj.to_ref_owned().ref_clone()),
                            None => ZVal::from(()),
                        };
                        let declaring_scope_zval = ZVal::from(class.clone());
                        let function_zval = ZVal::from(function.clone());
                        let filename_zval = ZVal::from(file.clone());
                        let lineno_zval = ZVal::from(line as i64);

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
                    }
                });
            }
        }
    }
}