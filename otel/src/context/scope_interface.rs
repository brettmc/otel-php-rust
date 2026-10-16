use phper::{
    classes::{InterfaceEntity},
    functions::{ReturnType},
    types::{ReturnTypeHint},
};
use crate::context::context_interface::CONTEXT_INTERFACE_NAME;

pub const SCOPE_INTERFACE_NAME: &str = r"OpenTelemetry\Context\ScopeInterface";

pub fn make_scope_interface() -> InterfaceEntity {
    let mut interface = InterfaceEntity::new(SCOPE_INTERFACE_NAME);

    // `detach()` returns zero for an expected call and a non-zero bitmask otherwise, so
    // userland needs these to interpret it -- without them the return value is an
    // uninterpretable integer. They are the same values upstream declares, computed the same
    // way (`1 << (PHP_INT_SIZE << 3) - N`) so a 32-bit build gets 32-bit constants rather
    // than silently disagreeing with PHP's own integer width.
    //
    // `DetachResult::to_php_flags` is what produces the values at runtime; the unit tests
    // there pin the two to each other.
    interface.add_constant("DETACHED", 1i64 << (i64::BITS - 1));
    interface.add_constant("INACTIVE", 1i64 << (i64::BITS - 2));
    interface.add_constant("MISMATCH", 1i64 << (i64::BITS - 3));

    interface
        .add_method("detach")
        .return_type(ReturnType::new(ReturnTypeHint::Int));

    interface
        .add_method("context")
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(CONTEXT_INTERFACE_NAME))));

    interface
}
