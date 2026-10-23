//! `ContextKey`, the identity under which a value is stored in a `Context`.
//!
//! A key's *name* is debugging aid only. Two keys with the same name are two different keys,
//! and the same key object is the only way to read back a value — matching upstream, which
//! looks keys up by `spl_object_id($key)` and never by name. `Context::createKey()` returning
//! a fresh object each call is therefore the point, not an inefficiency.

use phper::{
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, PropertyTypeHint, ReturnTypeHint},
};

pub const CONTEXT_KEY_CLASS_NAME: &str = r"OpenTelemetry\Context\ContextKey";
pub type ContextKeyClass = StateClass<()>;
pub type ContextKeyClassEntity = ClassEntity<()>;

pub fn make_context_key_class(context_key_interface: &Interface) -> ContextKeyClassEntity {
    let mut class = ClassEntity::<()>::new_with_default_state_constructor(CONTEXT_KEY_CLASS_NAME);

    class.set_final();
    class.implements(context_key_interface.clone());

    // Upstream holds the name in a promoted `private readonly ?string $name`. phper cannot
    // declare a promoted constructor property, so the constructor writes it instead.
    // Readonly from 8.1, typed from 8.0, a plain property on 7.x.
    class
        .add_readonly_property("name", Visibility::Private, PropertyTypeHint::String)
        .allow_null();

    class
        .add_method("__construct", Visibility::Public, |this, arguments| {
            // Declared with a default, so `new ContextKey()` is legal and yields an anonymous
            // key -- which is a perfectly good key, since lookup is by identity.
            let name = arguments.first().cloned().unwrap_or_default();
            this.set_property("name", name);
            Ok::<_, phper::Error>(())
        })
        .argument(
            Argument::new("name")
                .with_type_hint(ArgumentTypeHint::String)
                .allow_null()
                .with_default_value("null"),
        );

    class
        .add_method("name", Visibility::Public, |this, _| {
            Ok::<_, phper::Error>(this.get_property("name").clone())
        })
        .return_type(ReturnType::new(ReturnTypeHint::String).allow_null());

    class
}
