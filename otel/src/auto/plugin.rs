use std::sync::Arc;
use phper::{
    values::{ExecuteData, ZVal},
    objects::ZObj,
};

pub trait Plugin: Send + Sync {
    fn get_handlers(&self) -> &[Arc<dyn Handler + Send + Sync>];
    fn get_name(&self) -> &str;
}

pub trait Handler: Send + Sync {
    /// Should the function in execute data be observed by this plugin?
    fn get_targets(&self) -> Vec<(Option<String>, String)>;
    fn get_callbacks(&self) -> HandlerCallbacks;
}

pub struct HandlerCallbacks {
    pub pre_observe: Option<ObserverPreHook>,
    pub post_observe: Option<ObserverPostHook>,
}

pub type ObserverPreHook = Box<dyn Fn(&mut ExecuteData) + Send + Sync>;
// TODO Option<ZVal) for return value
pub type ObserverPostHook = Box<dyn Fn(&mut ExecuteData, &mut ZVal, Option<&mut ZObj>) + Send + Sync>;
pub type HandlerList = Vec<Arc<dyn Handler + Send + Sync>>;
pub type HandlerSlice = [Arc<dyn Handler + Send + Sync>];

pub struct FunctionObserver {
    pre_hooks: Vec<ObserverPreHook>,
    post_hooks: Vec<ObserverPostHook>,
}

impl FunctionObserver {
    pub fn new() -> Self {
        Self {
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
        }
    }

    pub fn pre_hooks(&self) -> &[ObserverPreHook] {
        &self.pre_hooks
    }

    pub fn post_hooks(&self) -> &[ObserverPostHook] {
        &self.post_hooks
    }

    pub fn add_pre_hook(&mut self, hook: ObserverPreHook) {
        self.pre_hooks.push(hook);
    }

    /// Adds a post-observe hook
    pub fn add_post_hook(&mut self, hook: ObserverPostHook) {
        self.post_hooks.push(hook);
    }

    /// Checks if this function has any hooks
    pub fn has_hooks(&self) -> bool {
        !self.pre_hooks.is_empty() || !self.post_hooks.is_empty()
    }
}