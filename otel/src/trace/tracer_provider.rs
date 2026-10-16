use phper::{
    classes::{ClassEntity, Interface, StateClass, Visibility},
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
    InstrumentationScope,
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
use crate::{
    provider::{self, ProviderKey},
    request,
    trace::{
        memory_exporter::MEMORY_EXPORTER,
        tracer::TracerClass,
        tracer_interface::TRACER_INTERFACE_NAME,
    },
    runtime::init_tokio_runtime,
    util,
};

const TRACER_PROVIDER_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\TracerProvider";

pub type TracerProviderClass = StateClass<()>;

static TRACER_PROVIDERS: LazyLock<Mutex<HashMap<ProviderKey, Arc<SdkTracerProvider>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static NOOP_TRACER_PROVIDER: LazyLock<Arc<SdkTracerProvider>> = LazyLock::new(|| {
    Arc::new(SdkTracerProvider::builder()
        .with_resource(Resource::builder_empty().build())
        .with_sampler(AlwaysOff)
        .build())
});

pub fn init_once() {
    let key = provider::provider_key();
    let mut providers = provider::lock_registry(&TRACER_PROVIDERS);
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

    let resource = provider::build_resource();

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
        let exporter = if env::var("OTEL_EXPORTER_OTLP_PROTOCOL").as_deref() == Ok("http/protobuf") {
            tracing::debug!("Using http/protobuf trace exporter");
            OtlpSpanExporter::builder()
                .with_http()
                .with_protocol(Protocol::HttpBinary)
                .build()
        } else {
            tracing::debug!("Using gRPC trace exporter with tokio runtime");
            match init_tokio_runtime() {
                Some(runtime) => runtime.block_on(async {
                    OtlpSpanExporter::builder()
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
                if use_simple_exporter {
                    builder = builder.with_simple_exporter(exporter);
                } else {
                    builder = builder.with_batch_exporter(exporter);
                }
            }
            // a misconfigured exporter must not take down the PHP process; fall
            // back to not exporting, and leave the config error in the log
            Err(err) => {
                tracing::error!("Failed to create OTLP trace exporter, tracing is disabled: {}", err);
                providers.insert(key, NOOP_TRACER_PROVIDER.clone());
                return;
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
    provider::get_provider(&TRACER_PROVIDERS, "Tracer", &NOOP_TRACER_PROVIDER)
}

/// Flush the current request's provider, reporting whether it succeeded.
pub fn force_flush() -> bool {
    provider::force_flush_registry(&TRACER_PROVIDERS, "Tracer", |p| p.force_flush())
}

pub fn shutdown() -> bool {
    provider::shutdown_registry(&TRACER_PROVIDERS, "Tracer", |p| p.shutdown())
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

            let tracer = provider.tracer_with_scope(scope);
            let mut object = tracer_class.init_object()?;
            *object.as_mut_state() = Some(tracer);
            Ok::<_, phper::Error>(object)
        })
        .argument(Argument::new("name").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("version").with_type_hint(ArgumentTypeHint::String).allow_null().with_default_value("null"))
        .argument(Argument::new("schemaUrl").with_type_hint(ArgumentTypeHint::String).allow_null().with_default_value("null"))
        .argument(Argument::new("attributes").with_type_hint(ArgumentTypeHint::Iterable).with_default_value("[]"))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(TRACER_INTERFACE_NAME))));

    // `forceFlush` and `shutdown` are declared on the class, not on
    // `API\Trace\TracerProviderInterface`: upstream's API interface carries only `getTracer`,
    // and these two belong to `SDK\Trace\TracerProviderInterface`, which this extension does
    // not expose. Upstream also takes a `?CancellationInterface $cancellation = null` on
    // both, which is omitted here: there is no cancellation type to name, and phper cannot
    // give a class-typed argument a default value in any case.
    class
        // The return type must be declared *and* the handler must return a real bool: an
        // internal function's return type is never verified by the engine, so a handler
        // returning Rust `()` under this hint would hand userland NULL silently.
        .add_method("forceFlush", Visibility::Public, |_, _| -> phper::Result<bool> {
            Ok(force_flush())
        })
        .return_type(ReturnType::new(ReturnTypeHint::Bool));

    class
        // The same `shutdown()` MSHUTDOWN runs, exposed to PHP so userland can flush and
        // release a provider before the request ends rather than waiting for teardown.
        .add_method("shutdown", Visibility::Public, |_, _| -> phper::Result<bool> {
            Ok(shutdown())
        })
        .return_type(ReturnType::new(ReturnTypeHint::Bool));

    class
}
