use crate::auto::plugin::{FunctionObserver, Plugin};
use crate::auto::plugins::{
    laminas::LaminasPlugin,
    psr18::Psr18Plugin,
    zf1::Zf1Plugin,
};
use phper::{
    classes::ClassEntry,
    functions::ZFunc,
    ini::ini_get,
    strings::{ZString},
    values::ExecuteData,
};
use std::{
    ffi::CStr,
    collections::HashSet,
};

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin + Send + Sync>>,
}

impl PluginManager {
    pub fn new() -> Self {
        tracing::debug!("PluginManager::init");
        // tracing::debug!("PluginManager::new");
        let mut manager = Self {plugins: vec![] };
        manager.init();
        manager
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

    pub fn get_function_observer(&self, execute_data: &mut ExecuteData) -> Option<FunctionObserver> {
        let mut observer = FunctionObserver::new();

        for plugin in &self.plugins {
            //tracing::trace!("plugin: {}", plugin.get_name());
            for handler in plugin.get_handlers() {
                if should_trace(execute_data.func(), &handler.get_targets(), plugin.get_name()) {
                    let callbacks = handler.get_callbacks();

                    if let Some(pre) = callbacks.pre_observe {
                        observer.add_pre_hook(Box::new(move |execute_data| {
                            pre(execute_data);
                        }));
                    }

                    if let Some(post) = callbacks.post_observe {
                        observer.add_post_hook(Box::new(move |execute_data, retval, exception| {
                            post(execute_data, retval, exception);
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
}

fn get_disabled_plugins() -> HashSet<String> {
    let value = ini_get::<Option<&CStr>>("otel.auto.disabled_plugins")
        .and_then(|cstr| cstr.to_str().ok())
        .unwrap_or("");
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn should_trace(func: &ZFunc, targets: &[(Option<String>, String)], plugin_name: &str) -> bool {
    let function_name: ZString = func.get_function_or_method_name();
    let function_name_str = match function_name.to_str() {
        Ok(name) => name,
        Err(_) => return false, // If the function name is not valid UTF-8, return false
    };
    let parts: Vec<&str> = function_name_str.split("::").collect();
    let is_method = parts.len() == 2;
    let observed_name_pair = if is_method {
        (Some(parts[0].to_string()), parts[1].to_string())
    } else {
        (None, function_name_str.to_string())
    };

    tracing::trace!("[plugin={}] should_trace: function_name: {:?}", plugin_name, function_name_str);
    if targets.iter().any(|target| target == &observed_name_pair) {
        //tracing::trace!("should_trace:: {:?} matches on name_pair", name_pair);
        return true;
    } else {
        //tracing::trace!("should_trace:: {:?} does not match on name_pair", name_pair);
    }

    //check for interfaces
    if !is_method {
        //tracing::trace!("[plugin={}] not checking interfaces, {} is not a class::method", plugin_name, function_name_str);
        return false;
    }

    let ce = match func.get_class() {
        Some(class_entry) => class_entry,
        None => return false,
    };
    for (target_class_name, target_method_name) in targets.iter() {
        if let Some(interface_name) = target_class_name {
            // Only check if the observed class is an instance of the interface
            match ClassEntry::from_globals(interface_name.to_string()) {
                Ok(iface_ce) => {
                    if ce.is_instance_of(&iface_ce) && &observed_name_pair.1 == target_method_name {
                        return true;
                    }
                }
                Err(_) => {}
            }
        }
    }

    false
}