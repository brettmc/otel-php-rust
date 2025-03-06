use std::sync::Arc;

pub trait Plugin: Send + Sync {
    /// Determines whether this plugin is enabled. Could be based on .ini config, or custom logic.
    fn is_enabled(&self) -> bool;
    fn get_handlers(&self) -> Vec<Arc<dyn Handler + Send + Sync>>;
}

pub trait Handler: Send + Sync {
    //fn get_functions(&self) -> Vec<String>;
    /// Should the function in execute data be observed by this plugin?
    //fn matches(&self, func: &str) -> bool; //TODO accept interfaces?
    fn get_functions(&self) -> Vec<String>;
    fn get_interfaces(&self) -> Vec<String>;
    fn get_callbacks(&self) -> HandlerCallbacks;
}

pub struct HandlerCallbacks {
    pub pre_observe: Option<unsafe extern "C" fn(*mut phper::sys::zend_execute_data)>,
    pub post_observe: Option<unsafe extern "C" fn(*mut phper::sys::zend_execute_data, *mut phper::sys::zval)>,
}

pub type ObserverHook = Box<dyn Fn(&mut phper::sys::zend_execute_data) + Send + Sync>;

pub struct FunctionObserver {
    pre_hooks: Vec<ObserverHook>,
    post_hooks: Vec<ObserverHook>,
}

impl FunctionObserver {
    pub fn new() -> Self {
        Self {
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
        }
    }

    pub fn pre_hooks(&self) -> &[ObserverHook] {
        &self.pre_hooks
    }

    pub fn post_hooks(&self) -> &[ObserverHook] {
        &self.post_hooks
    }

    pub fn add_pre_hook(&mut self, hook: ObserverHook) {
        self.pre_hooks.push(hook);
    }

    /// Adds a post-observe hook
    pub fn add_post_hook(&mut self, hook: ObserverHook) {
        self.post_hooks.push(hook);
    }

    /// Checks if this function has any hooks
    pub fn has_hooks(&self) -> bool {
        !self.pre_hooks.is_empty() || !self.post_hooks.is_empty()
    }
}
