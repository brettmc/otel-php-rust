use phper::{
    classes::{ClassEntity, Interface, StateClass, Visibility},
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
    InstrumentationScope,
    KeyValue,
    trace::TracerProvider,
};
use opentelemetry_stdout::SpanExporter as StdoutSpanExporter;
use opentelemetry_otlp::{
    Protocol,
    SpanExporter as OtlpSpanExporter,
    WithExportConfig,
};
use opentelemetry_sdk::{
    trace::{
        Sampler::AlwaysOff,
        SdkTracerProvider,
    },
    Resource,
};
use once_cell::sync::{
    Lazy,
    OnceCell,
};
use tokio::runtime::Runtime;
use crate::{
    request,
    trace::{
        memory_exporter::MEMORY_EXPORTER,
        tracer::TracerClass
    },
    util,
};

const TRACER_PROVIDER_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\TracerProvider";

pub type TracerProviderClass = StateClass<()>;

static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();
static TRACER_PROVIDERS: Lazy<Mutex<HashMap<(u32, String), Arc<SdkTracerProvider>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NOOP_TRACER_PROVIDER: Lazy<Arc<SdkTracerProvider>> = Lazy::new(|| {
    Arc::new(SdkTracerProvider::builder()
        .with_resource(Resource::builder_empty().build())
        .with_sampler(AlwaysOff)
        .build())
});

//tracer provider per (service_name, resource_attributes) pair (from .env, if enabled)
fn get_tracer_provider_key() -> (u32, String) {
    let pid = process::id();
    let service_name = env::var("OTEL_SERVICE_NAME").unwrap_or_default();
    let resource_attrs = env::var("OTEL_RESOURCE_ATTRIBUTES").unwrap_or_default();
    let key = format!("{}:{}", service_name, resource_attrs);
    (pid, key)
}

pub fn init_once() {
    let key = get_tracer_provider_key();
    let mut providers = TRACER_PROVIDERS.lock().unwrap();
    if providers.contains_key(&key) {
        tracing::debug!("tracer provider already exists for key {:?}", key);
        return;
    }
    tracing::debug!("creating tracer provider for key {:?}", key);
    let use_simple_exporter = env::var("OTEL_SPAN_PROCESSOR").as_deref() == Ok("simple");
    tracing::debug!("SpanProcessor={}", if use_simple_exporter {"simple"} else {"batch"});
    if env::var("OTEL_TRACES_EXPORTER").as_deref() == Ok("none") {
        tracing::debug!("Using no-op trace exporter");
        let provider = SdkTracerProvider::builder()
            .with_resource(Resource::builder_empty().build())
            .with_sampler(AlwaysOff)
            .build();
        providers.insert(key, Arc::new(provider.clone()));
        return;
    }

    let resource = Resource::builder()
        .with_attribute(KeyValue::new("telemetry.sdk.language", "php"))
        .with_attribute(KeyValue::new("telemetry.sdk.name", "ext-otel"))
        .with_attribute(KeyValue::new("telemetry.sdk.version", env!("CARGO_PKG_VERSION")))
        .with_attribute(KeyValue::new("process.runtime.name", util::get_sapi_module_name()))
        .with_attribute(KeyValue::new("process.runtime.version", util::get_php_version()))
        .with_attribute(KeyValue::new("process.pid", process::id().to_string()))
        .with_attribute(KeyValue::new("host.name", hostname::get().unwrap_or_default().to_string_lossy().to_string()))
        .build();

    let mut builder = SdkTracerProvider::builder();
    if env::var("OTEL_TRACES_EXPORTER").as_deref() == Ok("console") {
        tracing::debug!("Using Console trace exporter");
        let exporter = StdoutSpanExporter::default();
        if use_simple_exporter {
            builder = builder.with_simple_exporter(exporter);
        } else {
            builder = builder.with_batch_exporter(exporter);
        }
    } else if env::var("OTEL_TRACES_EXPORTER").as_deref() == Ok("memory") {
        tracing::debug!("Using in-memory test exporter");
        let exporter = MEMORY_EXPORTER.lock().unwrap().clone();
        if use_simple_exporter {
            builder = builder.with_simple_exporter(exporter);
        } else {
            builder = builder.with_batch_exporter(exporter);
        }
    } else {
        if env::var("OTEL_EXPORTER_OTLP_PROTOCOL").as_deref() == Ok("http/protobuf") {
            tracing::debug!("Using http/protobuf trace exporter");
            let exporter = OtlpSpanExporter::builder()
                .with_http()
                .with_protocol(Protocol::HttpBinary)
                .build()
                .expect("Failed to create OTLP http exporter");
            if use_simple_exporter {
                builder = builder.with_simple_exporter(exporter);
            } else {
                builder = builder.with_batch_exporter(exporter);
            }
        } else {
            tracing::debug!("Using gRPC trace exporter with tokio runtime");
            if TOKIO_RUNTIME.get().is_none() {
                tracing::debug!("Creating tokio runtime");
                let runtime = Runtime::new().expect("Failed to create Tokio runtime required for gRPC export");
                TOKIO_RUNTIME.set(runtime).expect("Tokio runtime already set");
                tracing::debug!("tokio runtime initialized");
            }
            let runtime = TOKIO_RUNTIME.get().expect("Tokio runtime not initialized");
            let exporter = runtime.block_on(async {
                OtlpSpanExporter::builder()
                    .with_tonic()
                    .build()
                    .expect("Failed to create OTLP grpc exporter")
            });
            if use_simple_exporter {
                builder = builder.with_simple_exporter(exporter);
            } else {
                builder = builder.with_batch_exporter(exporter);
            }
        }
    }
    let provider = Arc::new(builder
        .with_resource(resource)
        .build()
    );
    providers.insert(key, provider.clone());
}

pub fn get_tracer_provider() -> Arc<SdkTracerProvider> {
    if request::is_disabled() {
        tracing::debug!("OpenTelemetry is disabled for this request, returning no-op tracer provider");
        return NOOP_TRACER_PROVIDER.clone();
    }
    let providers = TRACER_PROVIDERS.lock().unwrap();
    let key = get_tracer_provider_key();
    if let Some(provider) = providers.get(&key) {
        return provider.clone();
    } else {
        tracing::warn!("no tracer provider initialized for key {:?}, using no-op", key);
        NOOP_TRACER_PROVIDER.clone()
    }
}

pub fn force_flush() {
    let pid = process::id();
    let providers = TRACER_PROVIDERS.lock().unwrap();
    let key = get_tracer_provider_key();
    if let Some(provider) = providers.get(&key) {
        tracing::info!("Flushing TracerProvider for pid {}", pid);
        match provider.force_flush() {
            Ok(_) => tracing::debug!("OpenTelemetry tracer provider flush success"),
            Err(err) => tracing::warn!("Failed to flush OpenTelemetry tracer provider: {:?}", err),
        }
    } else {
        tracing::info!("no tracer provider to flush for pid {}", pid);
    }
}

pub fn shutdown() {
    let pid = process::id();
    let mut providers = TRACER_PROVIDERS.lock().unwrap();
    let keys_to_remove: Vec<_> = providers
        .keys()
        .filter(|(k_pid, _)| *k_pid == pid)
        .cloned()
        .collect();
    if !keys_to_remove.is_empty() {
        tracing::info!("Shutting down all TracerProviders for pid {}", pid);
        for key in keys_to_remove {
            tracing::debug!("Shutting down TracerProvider for key {:?}", key);
            providers.remove(&key);
        }
    } else {
        tracing::info!("no tracer providers to shutdown for pid {}", pid);
    }
}

pub fn make_tracer_provider_class(
    tracer_class: TracerClass,
    tracer_provider_interface: Interface,
) -> ClassEntity<()> {
    let mut class =
        ClassEntity::<()>::new_with_default_state_constructor(TRACER_PROVIDER_CLASS_NAME);

    class.implements(tracer_provider_interface);
    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("getTracer", Visibility::Public, move |_this, arguments| {
            let provider = get_tracer_provider();
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

            let tracer = provider.tracer_with_scope(scope);
            let mut object = tracer_class.init_object()?;
            *object.as_mut_state() = Some(tracer);
            Ok::<_, phper::Error>(object)
        })
        .argument(Argument::new("name").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("version").optional().with_type_hint(ArgumentTypeHint::String).allow_null())
        .argument(Argument::new("schemaUrl").optional().with_type_hint(ArgumentTypeHint::String).allow_null())
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::ClassEntry(String::from("Iterable"))).with_default_value("[]"))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(r"OpenTelemetry\API\Trace\TracerInterface"))));

    class.add_method("forceFlush", Visibility::Public, |_, _| {
        force_flush();
        Ok::<_, Infallible>(())
    });

    class
}
