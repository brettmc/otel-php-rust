use phper::{
    classes::{ClassEntity, Visibility},
    functions::Argument,
    ini::{Policy},
    modules::Module,
    objects::StateObj,
    php_get_module,
    values::ZVal,
};
use opentelemetry::global::ObjectSafeSpan;
use opentelemetry::{global, trace::Tracer};
use opentelemetry::trace::{SpanKind, Status};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_stdout::SpanExporter;

#[php_get_module]
pub fn get_module() -> Module {
    let mut module = Module::new(
        env!("CARGO_CRATE_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
    );

    // register module ini
    module.add_ini("opentelemetry.enabled", true, Policy::All);
    module.add_ini("opentelemetry.min_trace_threshold", 100, Policy::All);

    // register hook functions
    module.on_module_init(|| {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let provider = TracerProvider::builder()
            .with_simple_exporter(SpanExporter::default())
            .build();
        global::set_tracer_provider(provider);
    });
    module.on_module_shutdown(|| {});
    module.on_request_init(|| {});
    module.on_request_shutdown(|| {});

    // register classes
    let mut globals_class = ClassEntity::new("OpenTelemetry\\Globals");
    globals_class.add_property("foo", Visibility::Private, 100);
    globals_class.add_method(
        "getFoo",
        Visibility::Public,
        |this: &mut StateObj<()>, _: &mut [ZVal]| {
            let prop = this.get_property("foo");
            Ok::<_, phper::Error>(prop.clone())
        },
    );
    globals_class
        .add_method(
            "setFoo",
            Visibility::Public,
            |this: &mut StateObj<()>, arguments: &mut [ZVal]| -> phper::Result<()> {
                this.set_property("foo", arguments[0].clone());
                Ok(())
            },
        )
        .argument(Argument::by_val("foo"));
    globals_class
        .add_method(
            "otel",
            Visibility::Public,
            |_,_| -> phper::Result<()> {
                let tracer = global::tracer("test");
                let mut span = tracer
                    .span_builder("test_span")
                    .with_kind(SpanKind::Server)
                    .start(&tracer);
                span.set_status(Status::Ok);
                Ok(())
            }
        );
    module.add_class(globals_class);

    // register extra info
    module.add_info("opentelemetry-rust version", "0.27.1");

    module
}