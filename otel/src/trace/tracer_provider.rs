use phper::{
    classes::{ClassEntity, StaticStateClass, Visibility},
};
use std::{
    convert::Infallible,
    sync::Arc,
};
//use opentelemetry::InstrumentationScope;
use opentelemetry::global::GlobalTracerProvider;
use opentelemetry_stdout::SpanExporter;
use opentelemetry::trace::TracerProvider;
use opentelemetry::{
    KeyValue,
};
use opentelemetry_sdk::trace::{
    SdkTracerProvider,
};
use opentelemetry_sdk::Resource;
use once_cell::sync::Lazy;
use crate::trace::tracer::TRACER_CLASS;

const TRACER_PROVIDER_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\TracerProvider";

pub static TRACER_PROVIDER_CLASS: StaticStateClass<Option<GlobalTracerProvider>> = StaticStateClass::null();

static TRACER_PROVIDER: Lazy<Arc<SdkTracerProvider>> = Lazy::new(|| {
    let resource = Resource::builder()
        .with_service_name("my_service_name")
        .with_attribute(KeyValue::new("telemetry.sdk.language", "php"))
        .with_attribute(KeyValue::new("telemetry.sdk.name", "ext-otel"))
        .with_attribute(KeyValue::new("telemetry.sdk.version", env!("CARGO_PKG_VERSION")))
        .build();
    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(SpanExporter::default())
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
        // let version = arguments[1].expect_z_str()?.to_str()?.to_string();
        // let schema_url = arguments[2].expect_z_str()?.to_str()?.to_string();
        // let scope = InstrumentationScope::builder(name)
        //     .with_version(version)
        //     .with_schema_url(schema_url)
        //     .build();
        let tracer = provider.tracer(name);
        let mut object = TRACER_CLASS.init_object()?;
        *object.as_mut_state() = Some(tracer);
        Ok::<_, phper::Error>(object)
    });

    class
}
