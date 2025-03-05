use phper::{
    sys,
    values::{
        ExecuteData,
    },
};
use std::{
    sync::Mutex,
};
use lazy_static::lazy_static;
use crate::{PluginManager};

lazy_static! {
    static ref PLUGIN_MANAGER: Mutex<Option<PluginManager>> = Mutex::new(None);
}

pub fn init(plugin_manager: PluginManager) {
    let mut manager_lock = PLUGIN_MANAGER.lock().unwrap();
    *manager_lock = Some(plugin_manager);
}

pub unsafe extern "C" fn observer_instrument(execute_data: *mut sys::zend_execute_data) -> sys::zend_observer_fcall_handlers {
    // println!("observer::observer_instrument");
    if let Some(exec_data) = ExecuteData::try_from_mut_ptr(execute_data) {
        let manager_lock = PLUGIN_MANAGER.lock().unwrap();
        if let Some(manager) = manager_lock.as_ref() {
            for plugin in manager.plugins() {
                for handler in plugin.get_handlers() {
                    if handler.matches(exec_data.func()) {
                        let callbacks = handler.get_callbacks();
                        return sys::zend_observer_fcall_handlers {
                            begin: callbacks.pre_observe,
                            end: callbacks.post_observe,
                        }
                    }
                }
            }
        } else {
            tracing::error!("Plugin manager not available");
        }
    }

    sys::zend_observer_fcall_handlers {
        begin: None,
        end: None,
    }
}

