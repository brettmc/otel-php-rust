use phper::{
    classes::{ClassEntity, StateClass, Visibility},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use std::{
    collections::HashMap,
    convert::Infallible,
    env,
    process,
    sync::{Arc, Mutex},
};
use opentelemetry::{
    logs::LoggerProvider,
    KeyValue,
    InstrumentationScope,
};
use opentelemetry_stdout::LogExporter as StdoutLogExporter;
use opentelemetry_sdk::{
    logs::{
        SimpleLogProcessor,
        BatchConfigBuilder,
        BatchLogProcessor,
        SdkLoggerProvider,
    },
    Resource,
};
use once_cell::sync::Lazy;
use crate::{
    logs::{
        logger::LoggerClass,
        memory_exporter::MEMORY_EXPORTER,
    },
    request,
    util,
    runtime::init_tokio_runtime,
};
use opentelemetry_otlp::{
    Protocol,
    LogExporter as OtlpLogExporter,
    WithExportConfig,
};

pub const LOGGER_PROVIDER_CLASS_NAME: &str = r"OpenTelemetry\API\Logs\LoggerProvider";

pub type LoggerProviderClass = StateClass<()>;

static LOGGER_PROVIDERS: Lazy<Mutex<HashMap<(u32, String), Arc<SdkLoggerProvider>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NOOP_LOGGER_PROVIDER: Lazy<Arc<SdkLoggerProvider>> = Lazy::new(|| {
    Arc::new(SdkLoggerProvider::builder()
        .with_resource(Resource::builder_empty().build())
        .build())
});

fn get_logger_provider_key() -> (u32, String) {
    let pid = process::id();
    let service_name = env::var("OTEL_SERVICE_NAME").unwrap_or_default();
    let resource_attrs = env::var("OTEL_RESOURCE_ATTRIBUTES").unwrap_or_default();
    let key = format!("{}:{}", service_name, resource_attrs);
    (pid, key)
}

pub fn init_once() {
    let key = get_logger_provider_key();
    let mut providers = LOGGER_PROVIDERS.lock().unwrap();
    if providers.contains_key(&key) {
        tracing::debug!("logger provider already exists for key {:?}", key);
        return;
    }
    tracing::debug!("creating logger provider for key {:?}", key);

    let resource = Resource::builder()
        .with_attribute(KeyValue::new("telemetry.sdk.language", "php"))
        .with_attribute(KeyValue::new("telemetry.sdk.name", "ext-otel"))
        .with_attribute(KeyValue::new("telemetry.sdk.version", env!("CARGO_PKG_VERSION")))
        .with_attribute(KeyValue::new("process.runtime.name", util::get_sapi_module_name()))
        .with_attribute(KeyValue::new("process.runtime.version", util::get_php_version()))
        .with_attribute(KeyValue::new("process.pid", process::id().to_string()))
        .with_attribute(KeyValue::new("host.name", hostname::get().unwrap_or_default().to_string_lossy().to_string()))
        .build();

    let mut builder = SdkLoggerProvider::builder().with_resource(resource);

    let exporter_type = env::var("OTEL_LOGS_EXPORTER").unwrap_or_else(|_| "otlp".to_string());
    let use_simple = env::var("OTEL_LOGS_PROCESSOR").as_deref() == Ok("simple");

    if exporter_type == "none" {
        tracing::debug!("Using no-op log exporter");
        // No exporter, just build the provider
    } else if exporter_type == "console" {
        let exporter = StdoutLogExporter::default();
        if use_simple {
            tracing::debug!("Using Simple log processor with Console exporter");
            builder = builder.with_log_processor(SimpleLogProcessor::new(exporter));
        } else {
            tracing::debug!("Using Batch log processor with Console exporter");
            let batch_config = BatchConfigBuilder::default().build();
            let batch = BatchLogProcessor::builder(exporter)
                .with_batch_config(batch_config)
                .build();
            builder = builder.with_log_processor(batch);
        }
    } else if exporter_type == "memory" {
        let exporter = MEMORY_EXPORTER.lock().unwrap().clone();
        if use_simple {
            tracing::debug!("Using Simple log processor with in-memory exporter");
            builder = builder.with_log_processor(SimpleLogProcessor::new(exporter));
        } else {
            tracing::debug!("Using Batch log processor with in-memory exporter");
            let batch_config = BatchConfigBuilder::default().build();
            let batch = BatchLogProcessor::builder(exporter)
                .with_batch_config(batch_config)
                .build();
            builder = builder.with_log_processor(batch);
        }
    } else {
        // Default to OTLP exporter
        if env::var("OTEL_EXPORTER_OTLP_PROTOCOL").as_deref() == Ok("http/protobuf") {
            tracing::debug!("Using http/protobuf log exporter");
            let exporter = OtlpLogExporter::builder()
                .with_http()
                .with_protocol(Protocol::HttpBinary)
                .build()
                .expect("Failed to create OTLP http log exporter");
            if use_simple {
                builder = builder.with_log_processor(SimpleLogProcessor::new(exporter));
            } else {
                let batch_config = BatchConfigBuilder::default().build();
                let batch = BatchLogProcessor::builder(exporter)
                    .with_batch_config(batch_config)
                    .build();
                builder = builder.with_log_processor(batch);
            }
        } else {
            tracing::debug!("Using gRPC log exporter with tokio runtime");
            let runtime = init_tokio_runtime();
            let exporter = runtime.block_on(async {
                OtlpLogExporter::builder()
                    .with_tonic()
                    .build()
                    .expect("Failed to create OTLP grpc log exporter")
            });
            if use_simple {
                builder = builder.with_log_processor(SimpleLogProcessor::new(exporter));
            } else {
                let batch_config = BatchConfigBuilder::default().build();
                let batch = BatchLogProcessor::builder(exporter)
                    .with_batch_config(batch_config)
                    .build();
                builder = builder.with_log_processor(batch);
            }
        }
    }

    let provider = Arc::new(builder.build());
    providers.insert(key, provider.clone());
}

pub fn get_logger_provider() -> Arc<SdkLoggerProvider> {
    if request::is_disabled() {
        tracing::debug!("OpenTelemetry is disabled for this request, returning no-op logger provider");
        return NOOP_LOGGER_PROVIDER.clone();
    }
    let providers = LOGGER_PROVIDERS.lock().unwrap();
    let key = get_logger_provider_key();
    if let Some(provider) = providers.get(&key) {
        return provider.clone();
    } else {
        tracing::warn!("no logger provider initialized for key {:?}, using no-op", key);
        NOOP_LOGGER_PROVIDER.clone()
    }
}

pub fn shutdown() {
    let pid = process::id();
    let mut providers = LOGGER_PROVIDERS.lock().unwrap();
    let keys_to_remove: Vec<_> = providers
        .keys()
        .filter(|(k_pid, _)| *k_pid == pid)
        .cloned()
        .collect();
    if !keys_to_remove.is_empty() {
        tracing::info!("Shutting down all LoggerProviders for pid {}", pid);
        for key in keys_to_remove {
            tracing::debug!("Shutting down LoggerProvider for key {:?}", key);
            providers.remove(&key);
        }
    } else {
        tracing::info!("no logger providers to shutdown for pid {}", pid);
    }
}

pub fn make_logger_provider_class(
    logger_class: LoggerClass,
    logger_provider_interface: phper::classes::Interface,
) -> ClassEntity<()> {
    let mut class =
        ClassEntity::<()>::new_with_default_state_constructor(LOGGER_PROVIDER_CLASS_NAME);

    class.implements(logger_provider_interface);
    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("getLogger", Visibility::Public, move |_this, arguments| {
            let provider = get_logger_provider();
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();

            let version = arguments.get(1)
                .and_then(|arg| arg.as_z_str())
                .map(|s| s.to_str().ok().map(|s| s.to_string()))
                .flatten();

            let schema_url = arguments.get(2)
                .and_then(|arg| arg.as_z_str())
                .map(|s| s.to_str().ok().map(|s| s.to_string()))
                .flatten();

            let attributes = arguments.get(3)
                .and_then(|arg| arg.as_z_arr())
                .map(|zarr| zarr.to_owned());

            let mut scope_builder = InstrumentationScope::builder(name);
            if let Some(version) = version {
                scope_builder = scope_builder.with_version(version);
            }
            if let Some(schema_url) = schema_url {
                scope_builder = scope_builder.with_schema_url(schema_url);
            }
            if let Some(attributes) = attributes {
                scope_builder = scope_builder.with_attributes(util::zval_arr_to_key_value_vec(attributes));
            }
            let scope = scope_builder.build();

            let logger = provider.logger_with_scope(scope);
            let mut object = logger_class.init_object()?;
            *object.as_mut_state() = Some(logger);

            Ok::<_, phper::Error>(object)
        })
        .argument(Argument::new("name").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("version").optional().with_type_hint(ArgumentTypeHint::String).allow_null())
        .argument(Argument::new("schemaUrl").optional().with_type_hint(ArgumentTypeHint::String).allow_null())
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::ClassEntry(String::from("Iterable"))).with_default_value("[]"))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\API\Logs\LoggerInterface"))));

    class
}
