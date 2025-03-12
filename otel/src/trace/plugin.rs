use std::sync::Arc;
use opentelemetry::{
    KeyValue,
    trace::SpanKind,
};

pub trait Plugin: Send + Sync {
    /// Determines whether this plugin is enabled. Could be based on .ini config, or custom logic.
    fn is_enabled(&self) -> bool;
    fn get_handlers(&self) -> Vec<Arc<dyn Handler + Send + Sync>>;
}

pub trait Handler: Send + Sync {
    /// Should the function in execute data be observed by this plugin?
    fn get_functions(&self) -> Vec<String>;
    fn get_interfaces(&self) -> Vec<String>;
    fn get_callbacks(&self) -> HandlerCallbacks;
}

pub struct HandlerCallbacks {
    pub pre_observe: Option<unsafe extern "C" fn(*mut phper::sys::zend_execute_data, &mut SpanDetails)>,
    pub post_observe: Option<unsafe extern "C" fn(*mut phper::sys::zend_execute_data, *mut phper::sys::zval)>,
}

pub type ObserverPreHook = Box<dyn Fn(&mut phper::sys::zend_execute_data, &mut SpanDetails) + Send + Sync>;
pub type ObserverPostHook = Box<dyn Fn(&mut phper::sys::zend_execute_data) + Send + Sync>;

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

pub struct SpanDetails {
    name: String,
    attributes: Vec<KeyValue>,
    kind: SpanKind,
}

impl SpanDetails {
    pub fn new(name: String, attributes: Vec<KeyValue>) -> Self {
        Self {name: name.clone(), attributes, kind: SpanKind::Internal }
    }
    pub fn add_attribute(&mut self, key: String, value: String) {
        self.attributes.push(KeyValue::new(key.clone(), value.clone()));
    }
    pub fn update_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
    pub fn set_kind(&mut self, kind: SpanKind) {
        self.kind = kind;
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn attributes(&self) -> Vec<KeyValue> {
        self.attributes.clone()
    }
    pub fn kind(&self) -> SpanKind {
        self.kind.clone()
    }
}