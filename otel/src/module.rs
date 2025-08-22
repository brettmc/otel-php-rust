use crate::{
    config,
    logging,
    util::get_sapi_module_name,
    auto,
    trace::tracer_provider,
};
use phper::ini::ini_get;
use once_cell::sync::OnceCell;
use tracing;

static DISABLED: OnceCell<bool> = OnceCell::new();

pub fn on_module_init() {
    logging::init_once();
    let cli_enabled = ini_get::<bool>(config::ini::OTEL_CLI_ENABLED);
    let sapi = get_sapi_module_name();
    let disabled = sapi == "cli" && !cli_enabled;
    DISABLED.set(disabled).ok();
    if disabled {
        tracing::debug!("OpenTelemetry::MINIT disabled");
        return;
    }
    tracing::debug!("OpenTelemetry::MINIT");

    let auto_enabled = ini_get::<bool>(config::ini::OTEL_AUTO_ENABLED);
    if auto_enabled {
        let plugin_manager = auto::plugin_manager::PluginManager::new();
        auto::plugin_manager::set_global(plugin_manager);

        #[cfg(otel_observer_supported)]
        {
            auto::observer::init();
        }
        #[cfg(otel_observer_not_supported)]
        {
            auto::execute::init();
        }
    } else {
        tracing::debug!("OpenTelemetry::MINIT auto-instrumentation disabled");
    }
}

pub fn on_module_shutdown() {
    if is_disabled() {
        return;
    }
    tracing::debug!("OpenTelemetry::MSHUTDOWN");
    tracer_provider::shutdown();
}

pub fn is_disabled() -> bool {
    *DISABLED.get().unwrap_or(&false)
}
