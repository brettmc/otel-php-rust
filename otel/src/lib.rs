use crate::{
    tracer::{make_tracer_class},
    globals::{make_globals_class},
};
use phper::{modules::Module, php_get_module};
use opentelemetry::{global};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_stdout::SpanExporter;

pub mod tracer;
pub mod globals;

#[php_get_module]
pub fn get_module() -> Module {
    let mut module = Module::new(
        env!("CARGO_CRATE_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
    );

    module.add_class(make_tracer_class());
    module.add_class(make_globals_class());

    module.on_module_init(|| {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let provider = TracerProvider::builder()
            .with_simple_exporter(SpanExporter::default())
            .build();
        global::set_tracer_provider(provider);
    });
    module.on_module_shutdown(|| {
        global::shutdown_tracer_provider();
    });

    module
}
