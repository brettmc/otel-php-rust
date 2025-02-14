use phper::{
    classes::{ClassEntity, StaticStateClass, Visibility},
};
use std::{
    convert::Infallible,
    env,
    sync::Arc,
};
use opentelemetry::{
    InstrumentationScope,
    global::GlobalTracerProvider,
    KeyValue,
    trace::TracerProvider,
    // trace::noop::NoopTracerProvider,
};
use opentelemetry_stdout::SpanExporter as StdoutSpanExporter;
use opentelemetry_otlp::{
    Protocol,
    SpanExporter as OtlpSpanExporter,
    WithExportConfig,
};
use opentelemetry_sdk::{
    trace::SdkTracerProvider,
    Resource,
};
use once_cell::sync::Lazy;
use crate::trace::tracer::TRACER_CLASS;
use crate::get_runtime;

const TRACER_PROVIDER_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\TracerProvider";

pub static TRACER_PROVIDER_CLASS: StaticStateClass<Option<GlobalTracerProvider>> = StaticStateClass::null();

static TRACER_PROVIDER: Lazy<Arc<SdkTracerProvider>> = Lazy::new(|| {
    // if env::var("OTEL_TRACES_EXPORTER").as_deref() == Ok("none") {
    //     let provider = NoopTracerProvider::new();
    //     return Arc::new(provider);
    // }
    let resource = Resource::builder()
        .with_service_name("my_service_name")
        .with_attribute(KeyValue::new("telemetry.sdk.language", "php"))
        .with_attribute(KeyValue::new("telemetry.sdk.name", "ext-otel"))
        .with_attribute(KeyValue::new("telemetry.sdk.version", env!("CARGO_PKG_VERSION")))
        .build();

    let mut builder = SdkTracerProvider::builder();
    if env::var("OTEL_TRACES_EXPORTER").as_deref() == Ok("console") {
        let exporter = StdoutSpanExporter::default();
        builder = builder.with_batch_exporter(exporter);
    } else {
        if env::var("OTEL_EXPORTER_OTLP_PROTOCOL").as_deref() == Ok("http/protobuf") {
            let exporter = OtlpSpanExporter::builder()
                .with_http()
                .with_protocol(Protocol::HttpBinary)
                .build()
                .expect("Failed to create OTLP http exporter");
            builder = builder.with_batch_exporter(exporter);
        } else {
            let exporter = get_runtime().block_on(async {
                OtlpSpanExporter::builder()
                    .with_tonic()
                    .build()
                    .expect("Failed to create OTLP grpc exporter")
            });
            builder = builder.with_batch_exporter(exporter);
        }
    }
    let provider = builder
        .with_resource(resource)
        .build();
    Arc::new(provider)
});

pub fn get_tracer_provider() -> &'static Arc<SdkTracerProvider> {
    &TRACER_PROVIDER
}

pub fn make_tracer_provider_class() -> ClassEntity<Option<GlobalTracerProvider>> {
    let mut class =
        ClassEntity::<Option<GlobalTracerProvider>>::new_with_default_state_constructor(TRACER_PROVIDER_CLASS_NAME);

    class.bind(&TRACER_PROVIDER_CLASS);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class.add_method("getTracer", Visibility::Public, |this, arguments| {
        let provider = this.as_state().as_ref().unwrap();
        let name = arguments[0].expect_z_str()?.to_str()?.to_string();
        //TODO implement (optional) version, schema_url, attributes
        // let version = arguments[1].expect_z_str()?.to_str()?.to_string();
        // let schema_url = arguments[2].expect_z_str()?.to_str()?.to_string();
        let scope = InstrumentationScope::builder(name)
        //     .with_version(version)
        //     .with_schema_url(schema_url)
             .build();
        let tracer = provider.tracer_with_scope(scope);
        let mut object = TRACER_CLASS.init_object()?;
        *object.as_mut_state() = Some(tracer);
        Ok::<_, phper::Error>(object)
    });

    class
}
