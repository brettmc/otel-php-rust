use phper::{
    classes::{ClassEntity, StateClass, Visibility},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use std::{
    collections::HashMap,
    convert::Infallible,
    env,
    sync::{Arc, LazyLock, Mutex},
};
use opentelemetry::{
    logs::LoggerProvider,
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
use crate::{
    logs::{
        logger::LoggerClass,
        logger_interface::LOGGER_INTERFACE_NAME,
        memory_exporter::MEMORY_EXPORTER,
    },
    provider::{self, ProviderKey},
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

static LOGGER_PROVIDERS: LazyLock<Mutex<HashMap<ProviderKey, Arc<SdkLoggerProvider>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static NOOP_LOGGER_PROVIDER: LazyLock<Arc<SdkLoggerProvider>> = LazyLock::new(|| {
    Arc::new(SdkLoggerProvider::builder()
        .with_resource(Resource::builder_empty().build())
        .build())
});

pub fn init_once() {
    let key = provider::provider_key();
    let mut providers = provider::lock_registry(&LOGGER_PROVIDERS);
    if providers.contains_key(&key) {
        tracing::debug!("logger provider already exists for key {:?}", key);
        return;
    }
    tracing::debug!("creating logger provider for key {:?}", key);

    let resource = provider::build_resource();

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
        let exporter = if env::var("OTEL_EXPORTER_OTLP_PROTOCOL").as_deref() == Ok("http/protobuf") {
            tracing::debug!("Using http/protobuf log exporter");
            OtlpLogExporter::builder()
                .with_http()
                .with_protocol(Protocol::HttpBinary)
                .build()
        } else {
            tracing::debug!("Using gRPC log exporter with tokio runtime");
            match init_tokio_runtime() {
                Some(runtime) => runtime.block_on(async {
                    OtlpLogExporter::builder()
                        .with_tonic()
                        .build()
                }),
                None => Err(opentelemetry_otlp::ExporterBuildError::InternalFailure(
                    "no tokio runtime available for gRPC export".to_string(),
                )),
            }
        };
        match exporter {
            Ok(exporter) => {
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
            // a misconfigured exporter must not take down the PHP process; fall
            // back to not exporting, and leave the config error in the log
            Err(err) => {
                tracing::error!("Failed to create OTLP log exporter, logging is disabled: {}", err);
                providers.insert(key, NOOP_LOGGER_PROVIDER.clone());
                return;
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
    provider::get_provider(&LOGGER_PROVIDERS, "Logger", &NOOP_LOGGER_PROVIDER)
}

/// Flush the current request's provider, reporting whether it succeeded.
pub fn force_flush() -> bool {
    provider::force_flush_registry(&LOGGER_PROVIDERS, "Logger", |p| p.force_flush())
}

pub fn shutdown() -> bool {
    provider::shutdown_registry(&LOGGER_PROVIDERS, "Logger", |p| p.shutdown())
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
                .and_then(|s| s.to_str().ok().map(|s| s.to_string()));

            let schema_url = arguments.get(2)
                .and_then(|arg| arg.as_z_str())
                .and_then(|s| s.to_str().ok().map(|s| s.to_string()));

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
        .argument(Argument::new("version").with_type_hint(ArgumentTypeHint::String).allow_null().with_default_value("null"))
        .argument(Argument::new("schemaUrl").with_type_hint(ArgumentTypeHint::String).allow_null().with_default_value("null"))
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::Iterable).with_default_value("[]"))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(LOGGER_INTERFACE_NAME))));

    // Declared on the class rather than on `API\Logs\LoggerProviderInterface`, for the reason
    // given in `tracer_provider::make_tracer_provider_class`.
    class
        .add_method("forceFlush", Visibility::Public, |_, _| -> phper::Result<bool> {
            Ok(force_flush())
        })
        .return_type(ReturnType::new(ReturnTypeHint::Bool));

    class
        .add_method("shutdown", Visibility::Public, |_, _| -> phper::Result<bool> {
            Ok(shutdown())
        })
        .return_type(ReturnType::new(ReturnTypeHint::Bool));

    class
}
