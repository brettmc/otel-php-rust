use phper::modules::Module;
use crate::{
    context::{
        context::{build_context_class, new_context_class},
        context_interface::make_context_interface,
        context_storage_interface::make_context_storage_interface,
        scope::{build_scope_class, new_scope_class},
        scope_interface::make_scope_interface,
        storage::{build_storage_class, new_storage_class},
        propagation::text_map_propagator_interface::make_text_map_propagator_interface,
    },
    trace::{
        local_root_span::make_local_root_span_class,
        memory_exporter::make_memory_exporter_class,
        non_recording_span::make_non_recording_span_class,
        span::make_span_class,
        span_interface::make_span_interface,
        span_builder::make_span_builder_class,
        status_code::make_status_code_interface,
        tracer::make_tracer_class,
        tracer_interface::make_tracer_interface,
        tracer_provider::{
            make_tracer_provider_class,
        },
        tracer_provider_interface::make_tracer_provider_interface,
        span_context::make_span_context_class,
        propagation::trace_context_propagator::make_trace_context_propagator_class,
    },
    globals::make_globals_class,
};

pub fn register_classes_and_interfaces(module: &mut Module) {
    // interfaces
    let scope_interface = module.add_interface(make_scope_interface());
    let context_interface = module.add_interface(make_context_interface());
    let context_storage_interface = module.add_interface(make_context_storage_interface());
    let tracer_interface = module.add_interface(make_tracer_interface());
    let tracer_provider_interface = module.add_interface(make_tracer_provider_interface());
    let text_map_propagator_interface = module.add_interface(make_text_map_propagator_interface());
    let span_interface = module.add_interface(make_span_interface());

    // co-dependent classes
    let mut scope_class_entity = new_scope_class();
    let mut context_class_entity = new_context_class();
    let mut storage_class_entity = new_storage_class();
    build_scope_class(&mut scope_class_entity, &context_class_entity, &scope_interface);
    build_context_class(&mut context_class_entity, &scope_class_entity, &storage_class_entity, context_interface);
    build_storage_class(&mut storage_class_entity, &scope_class_entity, &context_class_entity, &context_storage_interface);

    let trace_context_propagator_class = module.add_class(make_trace_context_propagator_class(text_map_propagator_interface, &context_class_entity));
    let span_context_class = module.add_class(make_span_context_class());
    let scope_class = module.add_class(scope_class_entity);
    let context_class = module.add_class(context_class_entity);
    let _storage_class = module.add_class(storage_class_entity);
    let _in_memory_exporter_class = module.add_class(make_memory_exporter_class());

    let span_class = module.add_class(make_span_class(scope_class.clone(), span_context_class.clone(), context_class.clone(), &span_interface));
    let non_recording_span_class = module.add_class(make_non_recording_span_class(scope_class.clone(), span_context_class.clone(), context_class.clone(), &span_interface));
    let span_builder_class = module.add_class(make_span_builder_class(span_class.clone()));
    let _local_root_span_class = module.add_class(make_local_root_span_class(span_class.clone(), non_recording_span_class.clone()));

    let tracer_class = module.add_class(make_tracer_class(span_builder_class.clone(), tracer_interface));
    let tracer_provider_class = module.add_class(make_tracer_provider_class(tracer_class.clone(), tracer_provider_interface));
    let _globals_class = module.add_class(make_globals_class(tracer_provider_class.clone(), trace_context_propagator_class.clone()));
    let _status_code_interface = module.add_interface(make_status_code_interface());
}
