use super::plugin::Plugin;
use crate::trace::plugins::{
    test::TestPlugin,
    psr18::Psr18Plugin,
};
use crate::trace::plugin::{FunctionObserver};
use phper::{
    classes::ClassEntry,
    functions::ZFunc,
    strings::{ZString},
    values::ExecuteData,
};

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
        self.plugins.push(Box::new(Psr18Plugin::new()));
        //#[cfg(feature="test")]
        self.plugins.push(Box::new(TestPlugin::new()));
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
                        observer.add_pre_hook(Box::new(move |execute_data, span_details| unsafe {
                            pre(execute_data, span_details);
                        }));
                    }

                    if let Some(post) = callbacks.post_observe {
                        observer.add_post_hook(Box::new(move |execute_data, span_ref, retval| unsafe {
                            post(execute_data, span_ref, retval);
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

fn should_trace(func: &ZFunc, functions: &[String], interfaces: &[String], plugin_name: &str) -> bool {
    let function_name: ZString = func.get_function_or_method_name();
    let function_name_str = match function_name.to_str() {
        Ok(name) => name,
        Err(_) => return false, // If the function name is not valid UTF-8, return false
    };
    tracing::trace!("[plugin={}] should_trace: function_name: {:?}", plugin_name, function_name_str);
    if functions.iter().any(|name| function_name_str == name) {
        //tracing::trace!("should_trace:: {:?} matches on function name", function_name_str);
        return true;
    } else {
        //tracing::trace!("should_trace:: {:?} does not match on function name", function_name_str);
    }

    //check for interfaces
    let parts: Vec<&str> = function_name_str.split("::").collect();
    if parts.len() != 2 {
        tracing::trace!("[plugin={}] not checking interfaces, {} is not a class::method", plugin_name, function_name_str);
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
        tracing::trace!("[plugin={}] interface={} method={}", plugin_name, interface_name, method_name);

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