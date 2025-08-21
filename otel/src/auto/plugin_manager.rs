use crate::{
    auto::{
        execute_data::get_fqn,
        plugin::{FunctionObserver, Plugin},
        plugins::{
            laminas::LaminasPlugin,
            psr18::Psr18Plugin,
            zf1::Zf1Plugin,
        },
    },
    config,
};
use phper::{
    classes::ClassEntry,
    functions::ZFunc,
    ini::ini_get,
    values::ExecuteData,
};
use once_cell::sync::OnceCell;
use std::{
    ffi::CStr,
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

static PLUGIN_MANAGER: OnceCell<RwLock<PluginManager>> = OnceCell::new();
static FUNCTION_OBSERVER_CACHE: OnceCell<RwLock<HashMap<String, Arc<FunctionObserver>>>> = OnceCell::new();

pub fn init_observer_cache() {
    FUNCTION_OBSERVER_CACHE.set(RwLock::new(HashMap::new())).ok();
}

pub fn set_global(manager: PluginManager) {
    PLUGIN_MANAGER.set(RwLock::new(manager)).ok();
}

pub fn get_global() -> Option<&'static RwLock<PluginManager>> {
    PLUGIN_MANAGER.get()
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin + Send + Sync>>,
}

impl PluginManager {
    pub fn new() -> Self {
        tracing::debug!("PluginManager::init");
        init_observer_cache();
        // tracing::debug!("PluginManager::new");
        let mut manager = Self {plugins: vec![] };
        manager.init();
        manager
    }

    /// calls request shutdown on all plugins, allowing them to do any post-request cleanup
    pub fn request_shutdown(&self) {
        tracing::debug!("PluginManager::request_shutdown");
        for plugin in &self.plugins {
            plugin.request_shutdown();
        }
    }

    fn init(&mut self) {
        let disabled = get_disabled_plugins();
        if !disabled.contains("laminas") {
            self.plugins.push(Box::new(LaminasPlugin::new()));
        }
        if !disabled.contains("psr18") {
            self.plugins.push(Box::new(Psr18Plugin::new()));
        }
        if !disabled.contains("zf1") {
            self.plugins.push(Box::new(Zf1Plugin::new()));
        }
        if !disabled.contains("test") {
            #[cfg(feature="test")]
            self.plugins.push(Box::new(crate::auto::plugins::test::TestPlugin::new()));
        }
    }

    pub fn plugins(&self) -> &Vec<Box<dyn Plugin + Send + Sync>> {
        &self.plugins
    }

    pub fn get_function_observer(&self, execute_data: &mut ExecuteData) -> Option<Arc<FunctionObserver>> {
        let fqn = get_fqn(execute_data)?;

        // Check cache
        if let Some(cache) = FUNCTION_OBSERVER_CACHE.get() {
            if let Some(observer) = cache.read().expect("Failed to acquire read lock on function observer cache").get(&fqn).cloned() {
                tracing::trace!("Using cached observer for function: {}", fqn);
                return Some(observer);
            }
        }

        // Build observer as before
        let mut observer = FunctionObserver::new();
        for plugin in &self.plugins {
            for handler in plugin.get_handlers() {
                if should_trace(execute_data.func(), &handler.get_targets(), plugin.get_name()) {
                    let callbacks = handler.get_callbacks();
                    if let Some(pre) = callbacks.pre_observe {
                        observer.add_pre_hook(Box::new(move |execute_data| {
                            pre(execute_data);
                        }));
                    }
                    if let Some(post) = callbacks.post_observe {
                        observer.add_post_hook(Box::new(move |execute_data, exception| {
                            post(execute_data, exception);
                        }));
                    }
                }
            }
        }

        if observer.has_hooks() {
            let arc_observer = Arc::new(observer);
            if let Some(cache) = FUNCTION_OBSERVER_CACHE.get() {
                tracing::trace!("Caching observer for function: {}", fqn);
                cache.write().expect("Failed to acquire write lock on function observer cache").insert(fqn, arc_observer.clone());
            }
            Some(arc_observer)
        } else {
            None
        }
    }
}

fn get_disabled_plugins() -> HashSet<String> {
    let value = ini_get::<Option<&CStr>>(config::ini::OTEL_AUTO_DISABLED_PLUGINS)
        .and_then(|cstr| cstr.to_str().ok())
        .unwrap_or("");
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn should_trace(func: &ZFunc, targets: &[(Option<String>, String)], _plugin_name: &str) -> bool {
    let name_zstr = func.get_function_or_method_name();
    let function_name = match name_zstr.to_str() {
        Ok(name) => name,
        Err(_) => return false,
    };

    let mut parts = function_name.splitn(2, "::");
    let class_part = parts.next();
    let method_part = parts.next();

    let observed_name_pair = if let Some(method) = method_part {
        (class_part.map(|s| s.to_owned()), method.to_owned())
    } else {
        (None, function_name.to_owned())
    };

    if targets.iter().any(|target| target == &observed_name_pair) {
        return true;
    }

    if observed_name_pair.0.is_none() {
        //tracing::trace!("[plugin={}] not checking interfaces, {} is not a class::method", plugin_name, function_name_str);
        return false;
    }

    let ce = match func.get_class() {
        Some(class_entry) => class_entry,
        None => return false,
    };
    for (target_class_name, target_method_name) in targets.iter() {
        if let Some(interface_name) = target_class_name {
            if &observed_name_pair.1 == target_method_name {
                if let Ok(iface_ce) = ClassEntry::from_globals(interface_name.clone()) {
                    if ce.is_instance_of(&iface_ce) {
                        return true;
                    }
                }
            }
        }
    }

    false
}