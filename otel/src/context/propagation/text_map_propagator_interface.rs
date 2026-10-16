use phper::{
    classes::InterfaceEntity,
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
};
use crate::context::{
    context_interface::CONTEXT_INTERFACE_NAME,
    propagation::{
        propagation_getter_interface::PROPAGATION_GETTER_INTERFACE_NAME,
        propagation_setter_interface::PROPAGATION_SETTER_INTERFACE_NAME,
    },
};

pub const TEXT_MAP_PROPAGATOR_INTERFACE_NAME: &str = r"OpenTelemetry\Context\Propagation\TextMapPropagatorInterface";

/// The arguments of `inject()`, shared with the concrete propagator.
///
/// Built once and used from both the interface and `TraceContextPropagator`, rather than
/// spelled out in each. Two copies of a signature drift, and a drifted copy is either a fatal
/// MINIT error (the class stops satisfying the interface) or, worse, a class declaration that
/// silently differs from the interface userland typed against.
pub fn inject_arguments() -> Vec<Argument> {
    vec![
        Argument::new("carrier")
            .with_type_hint(ArgumentTypeHint::Mixed)
            .by_ref(),
        // `.optional()` rather than `.with_default_value("null")`: both lower
        // `required_num_args` identically, but phper routes a class-typed argument through
        // `ZEND_ARG_OBJ_INFO`, which has no default-value field, so the default would be
        // silently dropped. `.optional()` says exactly what Zend ends up recording.
        //
        // The visible consequence is that the argument cannot be passed by name:
        // `inject($carrier, setter: $s)` raises ArgumentCountError for the skipped argument.
        Argument::new("setter")
            .with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                PROPAGATION_SETTER_INTERFACE_NAME,
            )))
            .allow_null()
            .optional(),
        Argument::new("context")
            .with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                CONTEXT_INTERFACE_NAME,
            )))
            .allow_null()
            .optional(),
    ]
}

/// The arguments of `extract()`, shared with the concrete propagator. See `inject_arguments`.
pub fn extract_arguments() -> Vec<Argument> {
    vec![
        // Untyped, as upstream: a carrier may be an array, a PSR-7 request, or anything the
        // getter understands.
        Argument::new("carrier"),
        Argument::new("getter")
            .with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                PROPAGATION_GETTER_INTERFACE_NAME,
            )))
            .allow_null()
            .optional(),
        Argument::new("context")
            .with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                CONTEXT_INTERFACE_NAME,
            )))
            .allow_null()
            .optional(),
    ]
}

pub fn make_text_map_propagator_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(TEXT_MAP_PROPAGATOR_INTERFACE_NAME);

    let inject = interface.add_method("inject");
    for argument in inject_arguments() {
        inject.argument(argument);
    }
    inject.return_type(ReturnType::new(ReturnTypeHint::Void));

    let extract = interface.add_method("extract");
    for argument in extract_arguments() {
        extract.argument(argument);
    }
    extract.return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
        CONTEXT_INTERFACE_NAME,
    ))));

    interface
}
