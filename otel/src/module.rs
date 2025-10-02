use crate::{
    config,
    logging,
    logs::logger_provider,
    util::get_sapi_module_name,
    auto,
    trace::tracer_provider,
};
use phper::ini::ini_get;
use once_cell::sync::OnceCell;
use tracing;
use phper::modules::Module;

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
    logger_provider::shutdown();
}

pub fn is_disabled() -> bool {
    *DISABLED.get().unwrap_or(&false)
}

pub fn add_module_info(module: &mut Module) {
    module.add_info("opentelemetry-rust", crate::OPENTELEMETRY_VERSION);
    module.add_info("phper", crate::PHPER_VERSION);
    module.add_info("tokio", crate::TOKIO_VERSION);

    //which auto-instrumentation mechanism is enabled
    #[cfg(otel_observer_supported)]
    {
        module.add_info("auto-instrumentation", "observer".to_string());
    }
    #[cfg(otel_observer_not_supported)]
    {
        module.add_info("auto-instrumentation", "zend_execute_ex".to_string());
    }
}

pub fn add_module_ini(module: &mut Module) {
    module.add_ini(config::ini::OTEL_LOG_LEVEL, "error".to_string(), phper::ini::Policy::All);
    module.add_ini(config::ini::OTEL_LOG_FILE, "/dev/stderr".to_string(), phper::ini::Policy::All);
    module.add_ini(config::ini::OTEL_CLI_CREATE_ROOT_SPAN, false, phper::ini::Policy::All);
    module.add_ini(config::ini::OTEL_CLI_ENABLED, false, phper::ini::Policy::All);
    module.add_ini(config::ini::OTEL_ENV_DOTENV_ENABLED, false, phper::ini::Policy::All);
    module.add_ini(config::ini::OTEL_ENV_SET_FROM_SERVER, false, phper::ini::Policy::All);
    module.add_ini(config::ini::OTEL_AUTO_ENABLED, true, phper::ini::Policy::All);
    module.add_ini(config::ini::OTEL_AUTO_DISABLED_PLUGINS, "".to_string(), phper::ini::Policy::All);
}
