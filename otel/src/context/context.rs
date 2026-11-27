use crate::context::{
    context_interface::CONTEXT_INTERFACE_NAME,
    context_key::ContextKeyClassEntity,
    context_key_interface::CONTEXT_KEY_INTERFACE_NAME,
    context_storage_interface::CONTEXT_STORAGE_INTERFACE_NAME,
    implicit_context_keyed_interface::IMPLICIT_CONTEXT_KEYED_INTERFACE_NAME,
    scope::ScopeClassEntity,
    scope_interface::SCOPE_INTERFACE_NAME,
    storage::{self, StorageClassEntity},
};
use opentelemetry::Context;
use phper::{
    alloc::ToRefOwned,
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
    values::ZVal,
};
use std::{convert::Infallible, sync::Arc};
use tracing::debug;

pub const CONTEXT_CLASS_NAME: &str = r"OpenTelemetry\Context\Context";
pub type ContextClass = StateClass<Option<Arc<Context>>>;
pub type ContextClassEntity = ClassEntity<Option<Arc<Context>>>;

/// The state of a Context is only populated by the factories on this class and on Storage.
/// An instance obtained any other way (`newInstanceWithoutConstructor()`) has none, which is
/// a userland mistake to report rather than a reason to panic across the FFI boundary.
fn uninitialized() -> phper::Error {
    phper::Error::boxed("Context was not created by a Context or Storage factory method")
}

pub fn new_context_class() -> ContextClassEntity {
    ClassEntity::<Option<Arc<Context>>>::new_with_default_state_constructor(CONTEXT_CLASS_NAME)
}

pub fn build_context_class(
    class: &mut ContextClassEntity,
    scope_class: &ScopeClassEntity,
    storage_class: &StorageClassEntity,
    context_key_class: &ContextKeyClassEntity,
    context_interface: Interface,
) {
    let context_class = class.bound_class();
    let scope_ce = scope_class.bound_class();
    let storage_ce = storage_class.bound_class();
    let context_key_class = context_key_class.bound_class();

    class.set_final();
    class.implements(context_interface);
    class.add_property("context_id", Visibility::Private, 0i64);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class.add_method("__destruct", Visibility::Public, |this, _| {
        let context_id = get_instance_id(this);
        debug!("Context::__destruct for context_id = {:?}", context_id);
        storage::maybe_remove_context_instance(context_id);
        Ok::<_, Infallible>(())
    });

    class
        .add_static_method("getCurrent", Visibility::Public, {
            let context_ce = context_class.clone();
            move |_| {
                let context = storage::current_context();

                let mut object = context_ce.init_object()?;
                *object.as_mut_state() = Some(context);
                // The id has to be the *active* context's, or the returned object cannot
                // reach that context's key/value pairs and
                // `Context::getCurrent()->get($key)` answers null inside a scope that set it.
                // 0 is correct only when nothing is attached: that is the root context, which
                // has no stored instance and therefore no values.
                let instance_id = storage::current_context_instance_id();
                object.set_property("context_id", instance_id.unwrap_or(0) as i64);
                Ok::<_, phper::Error>(object)
            }
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_INTERFACE_NAME,
        ))));

    // `with()` must return a *new* Context and leave the receiver unchanged, per the spec's
    // immutability contract -- which is also what makes a parent context safe to keep using
    // after deriving a child from it.
    //
    // Key/value pairs live in `storage`, keyed by the key object's handle to match upstream's
    // `spl_object_id($key)` lookup. See `storage::ContextValues` for why they cannot live in
    // the Rust SDK's own context.
    class
        .add_method("with", Visibility::Public, {
            let context_ce = context_class.clone();
            move |this, arguments| {
                let arc = this.as_state().as_ref().ok_or_else(uninitialized)?.clone();
                let key_handle = arguments[0].expect_mut_z_obj()?.handle();
                let key = arguments[0].clone();
                let value = arguments[1].clone();

                // The Rust `Context` is unchanged -- only the PHP-side map differs -- but a
                // distinct `context_id` is still needed, or the new object would share the
                // original's map and mutating one would change the other.
                let instance_id = storage::store_context_with_value(
                    arc.clone(),
                    get_instance_id(this),
                    key_handle,
                    key,
                    value,
                );

                let mut object = context_ce.init_object()?;
                *object.as_mut_state() = Some(arc);
                object.set_property("context_id", instance_id.unwrap_or(0) as i64);
                Ok::<_, phper::Error>(object)
            }
        })
        .argument(
            Argument::new("key").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                CONTEXT_KEY_INTERFACE_NAME,
            ))),
        )
        .argument(Argument::new("value").with_type_hint(ArgumentTypeHint::Mixed))
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_INTERFACE_NAME,
        ))));

    class
        // Upstream's `withContextValue` lets the *value* decide where it goes, which is how
        // `$context->withContextValue($span)` works without the caller knowing the span key.
        .add_method("withContextValue", Visibility::Public, |this, arguments| {
            let value = arguments[0].expect_mut_z_obj()?;
            let context = ZVal::from(this.to_ref_owned());
            // Delegates to the value, as upstream does, so a userland
            // ImplicitContextKeyedInterface works the same way the built-in Span does.
            let result = value.call("storeInContext", [context])?;
            Ok::<_, phper::Error>(result)
        })
        .argument(
            Argument::new("value").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                IMPLICIT_CONTEXT_KEYED_INTERFACE_NAME,
            ))),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_INTERFACE_NAME,
        ))));

    class
        // Returns null for an absent key, as upstream does, rather than the last value
        // written under any key.
        .add_method(
            "get",
            Visibility::Public,
            |this, arguments| -> phper::Result<ZVal> {
                // Confirm the state exists so an object built by `newInstanceWithoutConstructor()`
                // is still reported, even though the value lookup does not need it.
                let _ = this.as_state().as_ref().ok_or_else(uninitialized)?;
                let key_handle = arguments[0].expect_mut_z_obj()?.handle();
                Ok(
                    storage::get_context_value(get_instance_id(this), key_handle)
                        .unwrap_or_default(),
                )
            },
        )
        .argument(
            Argument::new("key").with_type_hint(ArgumentTypeHint::ClassEntry(String::from(
                CONTEXT_KEY_INTERFACE_NAME,
            ))),
        )
        .return_type(ReturnType::new(ReturnTypeHint::Mixed));

    class
        // Upstream's static factory, so userland never needs to name the ContextKey class.
        .add_static_method("createKey", Visibility::Public, {
            let context_key_ce = context_key_class.clone();
            move |arguments| {
                // `Context::createKey()` with no argument is legal, so the slot may be
                // absent and cannot be indexed unconditionally: a panic here aborts the
                // process rather than throwing.
                let name = arguments.first().cloned().unwrap_or_default();
                let object = context_key_ce.new_object([name])?;
                Ok::<_, phper::Error>(object)
            }
        })
        .argument(
            Argument::new("key")
                .with_type_hint(ArgumentTypeHint::String)
                .allow_null()
                .with_default_value("null"),
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_KEY_INTERFACE_NAME,
        ))));

    class
        .add_method("activate", Visibility::Public, {
            let scope_ce = scope_ce.clone();
            move |this, _arguments| {
                let arc = this.as_state().as_ref().ok_or_else(uninitialized)?.clone(); // Arc<Context>
                let previous_id = get_instance_id(this);
                let instance_id = storage::store_context_instance(arc.clone());
                debug!("Storing context: {:?}", instance_id);
                // Activating allocates a new context_id for the same context, so the PHP-side
                // keys have to come across or `$ctx->with($k, $v)->activate()` would lose
                // them, and `Context::getCurrent()->get($k)` inside the scope would be null.
                storage::inherit_context_values(previous_id, instance_id);

                this.set_property("context_id", instance_id.unwrap_or(0) as i64);
                storage::attach_context(instance_id).map_err(phper::Error::boxed)?;

                let mut object = scope_ce.init_object()?;
                object.set_property("context_id", instance_id.unwrap_or(0) as i64);

                Ok::<_, phper::Error>(object)
            }
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            SCOPE_INTERFACE_NAME,
        ))));

    class
        .add_static_method("storage", Visibility::Public, move |_| {
            let object = storage_ce.init_object()?;
            Ok::<_, phper::Error>(object)
        })
        // Upstream declares an *intersection*:
        // `ContextStorageInterface&ExecutionContextAwareInterface`. This extension does not
        // track fiber execution contexts, and phper cannot express an intersection in any
        // case, so this names the half that is true -- enough for everything downstream of
        // `Context::storage()->attach(...)` to be checked.
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_STORAGE_INTERFACE_NAME,
        ))));
}

/// Get context instance id from a PHP object
pub fn get_instance_id(obj: &phper::objects::ZObj) -> Option<u64> {
    obj.get_property("context_id")
        .as_long()
        .and_then(|id| if id > 0 { Some(id as u64) } else { None })
}
