//! Shared plumbing for the tracer and logger provider registries.
//!
//! Both signal types keep one SDK provider per (pid, service name + resource attributes)
//! pair and need the same resource, the same key, and the same shutdown/flush behaviour.
//! Keeping that here means the two cannot drift apart — notably the resource, where a
//! divergence would attribute a process's logs and spans to different entities.

use crate::util;
use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use std::{
    collections::HashMap,
    env, process,
    sync::{Arc, Mutex, MutexGuard},
};

/// (pid, "service_name:resource_attributes")
///
/// The pid is part of the key because a provider's exporter owns connections and background
/// threads that do not survive a fork, so a forked worker must build its own.
pub type ProviderKey = (u32, String);

/// The key identifying the provider the current request should use.
///
/// A provider is per service identity rather than global because a `.env` file can set
/// `OTEL_SERVICE_NAME`/`OTEL_RESOURCE_ATTRIBUTES` per request.
pub fn provider_key() -> ProviderKey {
    let pid = process::id();
    let service_name = env::var("OTEL_SERVICE_NAME").unwrap_or_default();
    let resource_attrs = env::var("OTEL_RESOURCE_ATTRIBUTES").unwrap_or_default();
    (pid, format!("{}:{}", service_name, resource_attrs))
}

/// The resource describing this PHP process, shared by every signal.
pub fn build_resource() -> Resource {
    Resource::builder()
        .with_attribute(KeyValue::new("telemetry.sdk.language", "php"))
        .with_attribute(KeyValue::new("telemetry.sdk.name", "ext-otel"))
        .with_attribute(KeyValue::new(
            "telemetry.sdk.version",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_attribute(KeyValue::new(
            "process.runtime.name",
            util::get_sapi_module_name(),
        ))
        .with_attribute(KeyValue::new(
            "process.runtime.version",
            util::get_php_version(),
        ))
        .with_attribute(KeyValue::new("process.pid", process::id().to_string()))
        .with_attribute(KeyValue::new(
            "host.name",
            hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        ))
        .build()
}

/// Lock a provider registry, recovering from a poisoned lock.
///
/// A poisoned lock means some earlier caller panicked while holding it. The map itself is
/// still structurally sound, and losing telemetry for the rest of the process (or
/// panicking in PHP's MSHUTDOWN) is worse than carrying on with it.
pub fn lock_registry<P>(
    registry: &Mutex<HashMap<ProviderKey, Arc<P>>>,
) -> MutexGuard<'_, HashMap<ProviderKey, Arc<P>>> {
    match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Remove and shut down every provider belonging to this process.
///
/// Shutting down explicitly rather than just dropping our `Arc` matters: the provider's
/// `Drop` only flushes if no clone handed out to a caller is still alive, and a failed
/// flush should be logged rather than silently dropping telemetry.
///
/// `signal` names the signal in those log messages. Every message here interpolates it as
/// `"{}Provider"`, so callers pass it capitalised ("Tracer", not "tracer") to keep one object
/// from appearing under two names in the logs.
///
/// Returns whether every provider shut down cleanly, which is what userland's
/// `shutdown(): bool` reports. Having nothing to shut down counts as success: the
/// postcondition the caller asked for already holds.
pub fn shutdown_registry<P>(
    registry: &Mutex<HashMap<ProviderKey, Arc<P>>>,
    signal: &str,
    shutdown: impl Fn(&P) -> Result<(), opentelemetry_sdk::error::OTelSdkError>,
) -> bool {
    let pid = process::id();
    let mut providers = lock_registry(registry);
    let keys: Vec<ProviderKey> = providers
        .keys()
        .filter(|(key_pid, _)| *key_pid == pid)
        .cloned()
        .collect();

    if keys.is_empty() {
        tracing::info!("no {}Providers to shutdown for pid {}", signal, pid);
        return true;
    }
    tracing::info!("Shutting down all {}Providers for pid {}", signal, pid);
    let mut all_succeeded = true;
    for key in keys {
        tracing::debug!("Shutting down {}Provider for key {:?}", signal, key);
        // Carry on through a failure rather than returning early: the remaining providers
        // still need shutting down, and this also runs from MSHUTDOWN.
        if let Some(provider) = providers.remove(&key)
            && let Err(err) = shutdown(&provider)
        {
            tracing::warn!(
                "Failed to shut down {}Provider for key {:?}: {}",
                signal,
                key,
                err
            );
            all_succeeded = false;
        }
    }

    all_succeeded
}

/// Flush the provider the current request would use, if one exists.
///
/// Returns whether the flush succeeded, which is what userland's `forceFlush(): bool`
/// reports. Having no provider to flush counts as success, for the same reason as in
/// [`shutdown_registry`]: there is no buffered telemetry left unflushed.
pub fn force_flush_registry<P>(
    registry: &Mutex<HashMap<ProviderKey, Arc<P>>>,
    signal: &str,
    force_flush: impl Fn(&P) -> Result<(), opentelemetry_sdk::error::OTelSdkError>,
) -> bool {
    let pid = process::id();
    let providers = lock_registry(registry);
    let key = provider_key();
    let Some(provider) = providers.get(&key) else {
        tracing::info!("no {}Provider to flush for pid {}", signal, pid);
        return true;
    };
    tracing::info!("Flushing {}Provider for pid {}", signal, pid);
    match force_flush(provider) {
        Ok(()) => {
            tracing::debug!("{}Provider flush success", signal);
            true
        }
        Err(err) => {
            tracing::warn!("Failed to flush {}Provider: {:?}", signal, err);
            false
        }
    }
}

/// Fetch the provider for the current request, or `fallback` if there is none.
pub fn get_provider<P>(
    registry: &Mutex<HashMap<ProviderKey, Arc<P>>>,
    signal: &str,
    fallback: &Arc<P>,
) -> Arc<P> {
    let providers = lock_registry(registry);
    let key = provider_key();
    match providers.get(&key) {
        Some(provider) => provider.clone(),
        None => {
            tracing::warn!(
                "no {}Provider initialized for key {:?}, using no-op",
                signal,
                key
            );
            fallback.clone()
        }
    }
}
