use crate::auto::plugin::{FunctionObserver, Plugin};
use crate::auto::plugins::{
    laminas::LaminasPlugin,
    psr18::Psr18Plugin,
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
                if should_trace(execute_data.func(), &handler.get_functions(), &handler.get_interfaces(), plugin.get_name()) {
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

fn should_trace(func: &ZFunc, functions: &[String], interfaces: &[String], plugin_name: &str) -> bool {
    let function_name: ZString = func.get_function_or_method_name();
    let function_name_str = match function_name.to_str() {
        Ok(name) => name,
        Err(_) => return false, // If the function name is not valid UTF-8, return false
    };
    //tracing::trace!("[plugin={}] should_trace: function_name: {:?}", plugin_name, function_name_str);
    if functions.iter().any(|name| function_name_str == name) {
        //tracing::trace!("should_trace:: {:?} matches on function name", function_name_str);
        return true;
    } else {
        //tracing::trace!("should_trace:: {:?} does not match on function name", function_name_str);
    }

    //check for interfaces
    let parts: Vec<&str> = function_name_str.split("::").collect();
    if parts.len() != 2 {
        //tracing::trace!("[plugin={}] not checking interfaces, {} is not a class::method", plugin_name, function_name_str);
        return false;
    }
    let _observed_class_name = parts[0];
    let observed_method_name = parts[1];

    let ce = match func.get_class() {
        Some(class_entry) => class_entry,
        None => return false,
    };
    for iface_entry in interfaces {
        let parts: Vec<&str> = iface_entry.split("::").collect();
        if parts.len() != 2 {
            tracing::warn!("[plugin={}] Skipping malformed interface entry: {}", plugin_name, iface_entry);
            continue;
        }
        let interface_name = parts[0];
        let method_name = parts[1];
        //tracing::trace!("[plugin={}] interface={} method={}", plugin_name, interface_name, method_name);

        match ClassEntry::from_globals(interface_name) {
            Ok(iface_ce) => {
                //tracing::trace!("interface CE found: {}", interface_name);
                if ce.is_instance_of(&iface_ce) {
                    //tracing::trace!("{} is an instance of {}", observed_class_name, interface_name);
                    if observed_method_name == method_name {
                        //tracing::trace!("methods match: {}", method_name);
                        return true;
                    }
                    //tracing::trace!("methods do not match: {}", method_name);
                } else {
                    //tracing::trace!{"{} is not an instance of {}", function_name_str, interface_name};
                }
            }
            Err(_) => {}
        }
    }

    false
}