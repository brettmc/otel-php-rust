use crate::{
    auto::{
        execute_data::get_function_and_class_name,
        plugin::{Handler, HandlerList, HandlerSlice, HandlerCallbacks, Plugin},
    },
};
use std::sync::Arc;
use std::collections::HashMap;
use phper::{
    values::{ExecuteData, ZVal},
    objects::ZObj,
};

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
        // Clear the HOOK_REGISTRY
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
        if let Ok((function, class)) = get_function_and_class_name(exec_data_ref) {
            if let Some(function) = function {
                HOOK_REGISTRY.with(|registry| {
                    if let Some((pre_hooks, _)) = registry.borrow().get(&(class.clone(), function.clone())) {
                        for mut pre_hook in pre_hooks.clone() {
                            if let Some(zobj) = pre_hook.as_mut_z_obj() {
                                let _ = zobj.call("__invoke", []);
                            }
                        }
                    }
                });
            }
        }
    }

    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        _retval: &mut ZVal,
        _exception: Option<&mut ZObj>
    ) {
        let exec_data_ref = unsafe { &mut *exec_data };
        if let Ok((function, class)) = get_function_and_class_name(exec_data_ref) {
            if let Some(function) = function {
                HOOK_REGISTRY.with(|registry| {
                    if let Some((_, post_hooks)) = registry.borrow().get(&(class.clone(), function.clone())) {
                        for mut post_hook in post_hooks.clone() {
                            if let Some(zobj) = post_hook.as_mut_z_obj() {
                                let _ = zobj.call("__invoke", []);
                            }
                        }
                    }
                });
            }
        }
    }
}