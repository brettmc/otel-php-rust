use super::plugin::Plugin;
use crate::trace::plugins::{
    test::TestPlugin,
};
use crate::trace::plugin::{FunctionObserver};
use phper::values::ExecuteData;
use phper::classes::ClassEntry;
use phper::strings::{ZString};
use phper::functions::ZFunc;

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

    pub fn get_function_observer(&self, execute_data: &mut ExecuteData) -> Option<FunctionObserver> {
        let mut observer = FunctionObserver::new();

        for plugin in &self.plugins {
            for handler in plugin.get_handlers() {
                if should_trace(execute_data.func(), &handler.get_functions(), &handler.get_interfaces()) {
                    let callbacks = handler.get_callbacks();

                    if let Some(pre) = callbacks.pre_observe {
                        observer.add_pre_hook(Box::new(move |execute_data, span_details| unsafe {
                            pre(execute_data, span_details);
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
}

fn should_trace(func: &ZFunc, functions: &[String], interfaces: &[String]) -> bool {
    let function_name: ZString = func.get_function_or_method_name();
    let function_name_str = match function_name.to_str() {
        Ok(name) => name,
        Err(_) => return false, // If the function name is not valid UTF-8, return false
    };
    // println!("function_name: {:?}", function_name);
    if functions.iter().any(|name| function_name_str == name) {
        return true;
    }

    //check for interfaces
    let ce = match func.get_class() {
        Some(class_entry) => class_entry,
        None => return false,
    };
    for iface_entry in interfaces {
        let parts: Vec<&str> = iface_entry.split("::").collect();
        if parts.len() != 2 {
            println!("Skipping malformed interface entry: {}", iface_entry);
            continue;
        }
        let interface_name = parts[0];
        let method_name = parts[1];

        match ClassEntry::from_globals(interface_name) {
            Ok(iface_ce) => {
                // println!("interface CE found: {}", interface_name);
                if ce.is_instance_of(&iface_ce) {
                    if iface_ce.has_method(method_name) {
                        // println!("match on interface + method");
                        return true;
                    }
                }
            }
            Err(_) => {}
        }
    }

    false
}