use phper::{
    classes::{ClassEntity, StateClass, Visibility},
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
use once_cell::sync::Lazy;
use crate::{
    trace::tracer::TracerClass,
    util,
};
use crate::get_runtime;

const TRACER_PROVIDER_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\TracerProvider";

pub type TracerProviderClass = StateClass<()>;

static TRACER_PROVIDERS: Lazy<Mutex<HashMap<u32, Arc<SdkTracerProvider>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn init_once() {
    let pid = process::id();
    let mut providers = TRACER_PROVIDERS.lock().unwrap();
    if providers.contains_key(&pid) {
        tracing::debug!("tracer provider already exists for pid {}", pid);
        return;
    }
    tracing::debug!("creating tracer provider for pid {}", pid);
    let use_simple_exporter = env::var("OTEL_SPAN_PROCESSOR").as_deref() == Ok("simple");
    tracing::debug!("SpanProcessor={}", if use_simple_exporter {"simple"} else {"batch"});
    if env::var("OTEL_TRACES_EXPORTER").as_deref() == Ok("none") {
        tracing::debug!("Using no-op trace exporter");
        let provider = SdkTracerProvider::builder()
            .with_resource(Resource::builder_empty().build())
            .with_sampler(AlwaysOff)
            .build();
        providers.insert(pid, Arc::new(provider.clone()));
        return;
    }
    let resource = Resource::builder()
        .with_attribute(KeyValue::new("telemetry.sdk.language", "php"))
        .with_attribute(KeyValue::new("telemetry.sdk.name", "ext-otel"))
        .with_attribute(KeyValue::new("telemetry.sdk.version", env!("CARGO_PKG_VERSION")))
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
            let runtime = get_runtime();
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
    providers.insert(pid, provider.clone());
}

pub fn get_tracer_provider() -> Arc<SdkTracerProvider> {
    let pid = process::id();
    let providers = TRACER_PROVIDERS.lock().unwrap();
    if let Some(provider) = providers.get(&pid) {
        return provider.clone();
    } else {
        tracing::error!("no tracer provider initialized for pid {}, using no-op", pid);
        Arc::new(SdkTracerProvider::builder()
            .with_resource(Resource::builder_empty().build())
            .with_sampler(AlwaysOff)
            .build()
        )
    }
}

pub fn force_flush() {
    let pid = process::id();
    let mut providers = TRACER_PROVIDERS.lock().unwrap();
    if providers.contains_key(&pid) {
        if let Some(provider) = providers.get(&pid) {
            tracing::info!("Flushing TracerProvider for pid {}", pid);
            match provider.force_flush() {
                Ok(_) => tracing::debug!("OpenTelemetry tracer provider flush success"),
                Err(err) => tracing::warn!("Failed to flush OpenTelemetry tracer provider: {:?}", err),
            }
            providers.remove(&pid);
            return;
        }
    }
    tracing::info!("no tracer provider to flush for pid {}", pid);
}

pub fn make_tracer_provider_class(tracer_class: TracerClass) -> ClassEntity<()> {
    let mut class =
        ClassEntity::<()>::new_with_default_state_constructor(TRACER_PROVIDER_CLASS_NAME);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class.add_method("getTracer", Visibility::Public, move |_this, arguments| {
        let provider = get_tracer_provider();
        let name = arguments[0].expect_z_str()?.to_str()?.to_string();

        let version = arguments.get(1)
            .and_then(|arg| arg.as_z_str()) // Check if it's a string
            .map(|s| s.to_str().ok().map(|s| s.to_string())) // Convert to Rust String
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
    });

    class.add_method("forceFlush", Visibility::Public, move |_, _| {
        let provider = get_tracer_provider();
        tracing::debug!("tracer_provider::force_flush");
        let result = match provider.force_flush() {
            Ok(_) => true,
            Err(err) => {
                tracing::warn!("force_flush failed: {}", err);
                false
            },
        };
        Ok::<_, phper::Error>(result)
    });

    class
}
