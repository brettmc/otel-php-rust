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

#[derive(Clone)]
pub struct RegisteredHook {
    pub class: Option<String>,
    pub function: String,
    pub pre: Option<ZVal>,
    pub post: Option<ZVal>,
}

// Thread-local registry for hooks
thread_local! {
    static HOOK_REGISTRY: std::cell::RefCell<HashMap<(Option<String>, String), RegisteredHook>> = std::cell::RefCell::new(HashMap::new());
}

pub fn add_hook(hook: RegisteredHook) {
    let key = (hook.class.clone(), hook.function.clone());
    HOOK_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(key, hook);
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
        tracing::debug!("HookHandler: pre_callback called");
        let exec_data_ref = unsafe { &mut *exec_data };
        match get_function_and_class_name(exec_data_ref) {
            Ok((function, class)) => {
                let function = match function {
                    Some(f) => f,
                    None => {
                        tracing::debug!("No function name found, exiting pre_callback");
                        return;
                    }
                };
                let pre_hook = HOOK_REGISTRY.with(|registry| {
                    registry
                        .borrow_mut()
                        .get_mut(&(class.clone(), function.clone()))
                        .and_then(|hook| hook.pre.as_mut().map(|z| z.clone()))
                });
                if let Some(mut pre_hook) = pre_hook {
                    tracing::debug!("Found pre callback for {:?}::{:?}", class, function);
                    if let Some(zobj) = pre_hook.as_mut_z_obj() {
                        let _ = zobj.call("__invoke", []);
                    } else {
                        tracing::warn!("Pre-hook is not a callable object");
                    }
                } else {
                    tracing::debug!("No hook registered for {:?}::{:?}", class, function);
                }
            }
            Err(e) => {
                tracing::debug!("Error getting function and class name: {:?}", e);
            }
        }
    }

    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        _retval: &mut ZVal,
        _exception: Option<&mut ZObj>
    ) {
        tracing::debug!("HookHandler: post_callback called");
        let exec_data_ref = unsafe { &mut *exec_data };
        match get_function_and_class_name(exec_data_ref) {
            Ok((function, class)) => {
                let function = match function {
                    Some(f) => f,
                    None => {
                        tracing::debug!("No function name found, exiting post_callback");
                        return;
                    }
                };
                let post_hook = HOOK_REGISTRY.with(|registry| {
                    registry
                        .borrow_mut()
                        .get_mut(&(class.clone(), function.clone()))
                        .and_then(|hook| hook.post.as_mut().map(|z| z.clone()))
                });
                if let Some(mut post_hook) = post_hook {
                    tracing::debug!("Found post callback for {:?}::{:?}", class, function);
                    if let Some(zobj) = post_hook.as_mut_z_obj() {
                        let _ = zobj.call("__invoke", []);
                    } else {
                        tracing::warn!("Post-hook is not a callable object");
                    }
                } else {
                    tracing::debug!("No post hook registered for {:?}::{:?}", class, function);
                }
            }
            Err(e) => {
                tracing::debug!("Error getting function and class name: {:?}", e);
            }
        }
    }
}