use super::plugin::Plugin;
use crate::trace::plugins::{
    test::TestPlugin,
};
use crate::trace::plugin::{FunctionObserver};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

static FUNCTION_OBSERVERS: OnceLock<RwLock<HashMap<String, FunctionObserver>>> = OnceLock::new();

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin + Send + Sync>>,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut manager = Self {plugins: vec![] };
        manager.init();
        manager
    }

    fn init(&mut self) {
        //#[cfg(feature="test")]
        self.plugins.push(Box::new(TestPlugin::new()));
    }

    pub fn plugins(&self) -> &Vec<Box<dyn Plugin + Send + Sync>> {
        &self.plugins
    }

    pub fn get_function_observer(&self, fqn: &str) -> Option<FunctionObserver> {
        let mut observer = FunctionObserver::new();

        for plugin in &self.plugins {
            for handler in plugin.get_handlers() {
                if handler.matches(fqn) {
                    let callbacks = handler.get_callbacks();

                    if let Some(pre) = callbacks.pre_observe {
                        observer.add_pre_hook(Box::new(move |execute_data| unsafe {
                            pre(execute_data);
                        }));
                    }

                    if let Some(post) = callbacks.post_observe {
                        observer.add_post_hook(Box::new(move |execute_data| unsafe {
                            post(execute_data, std::ptr::null_mut());
                        }));
                    }
                }
            }
        }

        if observer.has_hooks() {
            Some(observer)
        } else {
            None
        }
    }

    pub fn register_function_observer(&self, fqn: &str) {
        let mut lock = FUNCTION_OBSERVERS.get_or_init(|| RwLock::new(HashMap::new())).write().unwrap();

        if let Some(observer) = self.get_function_observer(fqn) {
            lock.insert(fqn.to_string(), observer);
        }
    }
}
