use crate::{
    auto::{
        execute_data::get_fqn,
        plugin::{FunctionObserver, Plugin},
        plugin::{
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

/// Memoizes the result of matching a function against every plugin's handlers, which is a
/// comparatively expensive scan involving `is_instance_of` checks.
///
/// This is the single cache of fqn -> observer: the observer backends look the observer up
/// here rather than keeping their own copy, so there is no second map to fall out of step.
///
/// Keyed on the fully-qualified name rather than the `zend_function *`, deliberately: a
/// closure or `eval`'d function is freed mid-request and its address can be reused by an
/// unrelated function, which would make a pointer-keyed entry return the wrong hooks.
static FUNCTION_OBSERVER_CACHE: OnceCell<RwLock<HashMap<String, Arc<FunctionObserver>>>> = OnceCell::new();

pub fn init_observer_cache() {
    FUNCTION_OBSERVER_CACHE.set(RwLock::new(HashMap::new())).ok();
}

/// Look up an already-built observer for `fqn`.
///
/// Returns `None` both when the function has never been matched and when it was matched and
/// found to have no hooks, which is what the observer backends want in either case.
pub fn get_cached_observer(fqn: &str) -> Option<Arc<FunctionObserver>> {
    let cache = FUNCTION_OBSERVER_CACHE.get()?;
    // A poisoned lock means a reader panicked while holding it; the map itself is still
    // consistent (entries are only ever inserted), so keep instrumenting.
    let guard = match cache.read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.get(fqn).cloned()
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
        let candidates: Vec<Box<dyn Plugin + Send + Sync>> = vec![
            Box::new(LaminasPlugin::new()),
            Box::new(Psr18Plugin::new()),
            Box::new(Zf1Plugin::new()),
            #[cfg(feature = "test")]
            Box::new(crate::auto::plugin::test::TestPlugin::new()),
        ];
        // Filtering on the plugin's own name means a new plugin cannot drift from the
        // key users disable it by; adding one is a single line here.
        self.plugins = candidates
            .into_iter()
            .filter(|plugin| {
                let name = plugin.get_name();
                if disabled.contains(name) {
                    tracing::debug!("PluginManager::plugin '{}' is disabled", name);
                    false
                } else {
                    true
                }
            })
            .collect();
    }

    pub fn plugins(&self) -> &Vec<Box<dyn Plugin + Send + Sync>> {
        &self.plugins
    }

    pub fn get_function_observer(&self, execute_data: &mut ExecuteData) -> Option<Arc<FunctionObserver>> {
        let fqn = get_fqn(execute_data)?;

        // Check cache
        if let Some(observer) = get_cached_observer(&fqn) {
            tracing::trace!("Using cached observer for function: {}", fqn);
            return Some(observer);
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
                        observer.add_post_hook(Box::new(move |execute_data, retval, exception| {
                            post(execute_data, retval, exception);
                        }));
                    }
                }
            }
        }

        if !observer.has_hooks() {
            return None;
        }
        let arc_observer = Arc::new(observer);
        if let Some(cache) = FUNCTION_OBSERVER_CACHE.get() {
            tracing::trace!("Caching observer for function: {}", fqn);
            match cache.write() {
                Ok(mut guard) => { guard.insert(fqn, arc_observer.clone()); }
                Err(poisoned) => { poisoned.into_inner().insert(fqn, arc_observer.clone()); }
            }
        }
        Some(arc_observer)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

fn get_disabled_plugins() -> HashSet<String> {
    let value = ini_get::<Option<&CStr>>(config::ini::OTEL_AUTO_DISABLED_PLUGINS)
        .and_then(|cstr| cstr.to_str().ok())
        .unwrap_or("");
    parse_disabled_plugins(value)
}

/// Parse the comma-separated `otel.auto.disabled_plugins` list into the set of names to
/// match against each plugin's `get_name()`.
///
/// Entries are trimmed because the ini value is hand-written and `a, b` is the natural way
/// to write it. Matching is otherwise exact: a name has to be spelled as the plugin spells
/// it, so there is no near-miss that silently leaves instrumentation enabled.
fn parse_disabled_plugins(value: &str) -> HashSet<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn should_trace(func: &ZFunc, targets: &[(Option<&'static str>, &'static str)], _plugin_name: &str) -> bool {
    let name_zstr = func.get_function_or_method_name();
    let function_name = match name_zstr.to_str() {
        Ok(name) => name,
        Err(_) => return false,
    };

    let mut parts = function_name.splitn(2, "::");
    let class_part = parts.next();
    let method_part = parts.next();

    let observed_name_pair = if let Some(method) = method_part {
        (class_part, method)
    } else {
        (None, function_name)
    };

    if targets.iter().any(|target| target.0 == observed_name_pair.0 && target.1 == observed_name_pair.1) {
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
        if let Some(interface_name) = target_class_name
            && target_method_name == &observed_name_pair.1
            && let Ok(iface_ce) = ClassEntry::from_globals(interface_name)
            && ce.is_instance_of(iface_ce) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_plugin_list() {
        let disabled = parse_disabled_plugins("laminas,psr18");
        assert!(disabled.contains("laminas"));
        assert!(disabled.contains("psr18"));
        assert_eq!(disabled.len(), 2);
    }

    #[test]
    fn whitespace_around_entries_is_ignored() {
        // The spelling used in tests/auto/autoinstrument-class-when-disabled.phpt.
        let disabled = parse_disabled_plugins("test,laminas, psr18");
        assert!(disabled.contains("psr18"));
        assert!(disabled.contains("laminas"));
        assert!(disabled.contains("test"));
    }

    #[test]
    fn empty_entries_are_dropped() {
        assert!(parse_disabled_plugins("").is_empty());
        assert!(parse_disabled_plugins(" , ,").is_empty());
        assert_eq!(parse_disabled_plugins("zf1,,").len(), 1);
    }

    #[test]
    fn names_are_matched_exactly() {
        // A near-miss must not disable a plugin: the failure mode of a loose match is
        // silently losing instrumentation the user still expects.
        let disabled = parse_disabled_plugins("not-zf1,zf1x,PSR18");
        assert!(!disabled.contains("zf1"));
        assert!(!disabled.contains("psr18"));
    }
}
