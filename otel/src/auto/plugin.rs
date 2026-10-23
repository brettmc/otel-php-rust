use phper::{
    objects::ZObj,
    values::{ExecuteData, ZVal},
};
use std::sync::Arc;

// Submodules
pub mod db;
pub mod laminas;
pub mod psr18;
#[cfg(feature = "test")]
pub mod test;
pub mod zf1;

// Plugin trait and related types
pub trait Plugin: Send + Sync {
    fn get_handlers(&self) -> &[Arc<dyn Handler + Send + Sync>];
    fn get_name(&self) -> &str;
    fn request_shutdown(&self) {
        // Default implementation does nothing
    }
}

/// A function or method a [`Handler`] wants to observe.
///
/// A target with a `class` matches two ways: the exact `Class::method` name, and any method
/// of that name on a class implementing or extending `class`. That is deliberate — most
/// targets name an interface (`Psr\Http\Client\ClientInterface::sendRequest`) that userland
/// implements under its own class name, so matching only the literal name would observe
/// nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    /// The class or interface the function belongs to, or `None` for a free function.
    pub class: Option<&'static str>,
    pub function: &'static str,
}

impl Target {
    /// A free function, e.g. `phpversion`.
    pub const fn function(function: &'static str) -> Self {
        Self {
            class: None,
            function,
        }
    }

    /// A method on `class`, matched on `class` itself or on any of its implementors.
    pub const fn method(class: &'static str, function: &'static str) -> Self {
        Self {
            class: Some(class),
            function,
        }
    }
}

pub trait Handler: Send + Sync {
    /// The functions this handler wants to observe.
    ///
    /// Returns a `'static` slice rather than an owned `Vec` because targets are fixed at
    /// compile time, and this is consulted once per distinct function the process executes:
    /// allocating a `Vec` per handler per call was pure overhead.
    fn get_targets(&self) -> &'static [Target];
    fn get_callbacks(&self) -> HandlerCallbacks;
}

/// The hooks a handler wants run around each of its [`Target`]s.
///
/// Both are optional: a handler may only need to open a span (pre), only read a return value
/// (post), or both. These are called by Rust — the engine's own `extern "C"` entry points are
/// in the backend modules (`auto::observer` / `auto::execute`), which catch panics before
/// anything here runs. Handler callbacks are still `unsafe` because they take the raw
/// `*mut ExecuteData` the engine handed us, but they carry no C ABI.
pub struct HandlerCallbacks {
    pub pre_observe: Option<ObserverPreHook>,
    pub post_observe: Option<ObserverPostHook>,
}

pub type ObserverPreHook = Box<dyn Fn(&mut ExecuteData) + Send + Sync>;
pub type ObserverPostHook =
    Box<dyn Fn(&mut ExecuteData, &mut ZVal, Option<&mut ZObj>) + Send + Sync>;
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

impl Default for FunctionObserver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_method_target_carries_its_class() {
        let target = Target::method(r"Psr\Http\Client\ClientInterface", "sendRequest");
        assert_eq!(target.class, Some(r"Psr\Http\Client\ClientInterface"));
        assert_eq!(target.function, "sendRequest");
    }

    #[test]
    fn a_function_target_has_no_class() {
        // `None` is what tells `should_trace` to skip the class-table lookup, so it has to
        // stay distinguishable from a method on an empty-named class.
        let target = Target::function("phpversion");
        assert_eq!(target.class, None);
        assert_eq!(target.function, "phpversion");
    }

    #[test]
    fn targets_compare_on_both_parts() {
        assert_eq!(Target::method("A", "run"), Target::method("A", "run"));
        assert_ne!(Target::method("A", "run"), Target::method("B", "run"));
        assert_ne!(Target::method("A", "run"), Target::method("A", "walk"));
        // A free function named `run` is not the same target as `A::run`.
        assert_ne!(Target::function("run"), Target::method("A", "run"));
    }

    #[test]
    fn targets_are_usable_in_a_const() {
        // Each handler stores its targets in a `const`, which only compiles while the
        // constructors stay `const fn`. Pin that, since losing it is a silent perf
        // regression back to allocating per call.
        const TARGETS: &[Target] = &[Target::method("A", "run"), Target::function("b")];
        assert_eq!(TARGETS.len(), 2);
    }
}
