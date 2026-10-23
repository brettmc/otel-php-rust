use crate::{
    context::{
        context::{ContextClassEntity, get_instance_id},
        context_interface::CONTEXT_INTERFACE_NAME,
        storage,
    },
    trace::local_root_span,
};
use phper::{
    arrays::ZArray,
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::{Argument, ReturnType},
    objects::ZObj,
    types::{ArgumentTypeHint, ReturnTypeHint},
    values::ZVal,
};
use std::convert::Infallible;

const SCOPE_CLASS_NAME: &str = r"OpenTelemetry\Context\Scope";
pub type ScopeClass = StateClass<()>;
pub type ScopeClassEntity = ClassEntity<()>;

pub fn new_scope_class() -> ScopeClassEntity {
    ScopeClassEntity::new_with_default_state_constructor(SCOPE_CLASS_NAME)
}

/// Read the scope's scratch map, creating it on first use.
///
/// `add_property` only accepts a `Scalar`, so the property cannot be declared as an empty
/// array; it starts as null and becomes an array the first time something is written. That
/// also matches how it reads: an untouched scope has nothing in it either way.
fn local_storage(this: &mut ZObj) -> &mut ZVal {
    let property = this.get_mut_property("localStorage");
    if property.as_z_arr().is_none() {
        *property = ZVal::from(ZArray::new());
    }
    property
}

/// Turn an `ArrayAccess` offset into the string key the scratch map uses.
///
/// Upstream narrows `$offset` to string in `ContextStorageScopeInterface`, but `ArrayAccess`
/// itself declares it `mixed` and an implementation may not narrow an inherited parameter,
/// so the coercion happens here rather than in the declaration.
fn offset_key(offset: &ZVal) -> phper::Result<String> {
    Ok(offset.expect_z_str()?.to_str()?.to_string())
}

pub fn build_scope_class(
    class: &mut ScopeClassEntity,
    context_class: &ContextClassEntity,
    context_storage_scope_interface: &Interface,
) {
    let _scope_class = class.bound_class();
    let context_ce = context_class.bound_class();
    // `ContextStorageScopeInterface`, not `ScopeInterface`: a Scope comes from a storage, so
    // it can answer `context()`, which upstream declares on the narrower interface rather
    // than promising it of every scope. The interface extends
    // `ScopeInterface` and `ArrayAccess`, so implementing it also obliges the four
    // offset* methods below.
    class.implements(context_storage_scope_interface.clone());
    class.add_property("context_id", Visibility::Private, 0i64);
    // Upstream's `ContextStorageNode` keeps a `private array $localStorage` for the same job:
    // somewhere instrumentation can hang per-scope bookkeeping without needing its own map
    // keyed by scope.
    class.add_property("localStorage", Visibility::Private, ());

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        // The handler must return a real `i64`, not Rust `()`: PHP does not enforce an
        // internal function's return type, so a unit return under this `: int` hint would
        // hand userland NULL where the spec promises zero-or-a-bitmask. `detach_context`
        // reports which condition fired so the bitmask can distinguish "not top-most" from
        // "stack empty" rather than collapsing both into a bool.
        .add_method(
            "detach",
            Visibility::Public,
            |this, _| -> phper::Result<i64> {
                let instance_id = get_instance_id(this);
                let result = storage::detach_context(instance_id);
                // Only tear down the local root span if the scope really was detached. An
                // out-of-order detach is refused (see `storage::detach_context`), and clearing
                // the local root for a scope that is still active would drop its context while
                // userland can still reach it through the still-active inner scope.
                if result.is_detached() {
                    local_root_span::maybe_remove_local_root_span(instance_id);
                }
                Ok(result.to_php_flags())
            },
        )
        .return_type(ReturnType::new(ReturnTypeHint::Int));

    class
        .add_method("context", Visibility::Public, move |this, _| {
            let instance_id = get_instance_id(this);
            let ctx = storage::get_context_instance(instance_id);
            let mut object = context_ce.init_object()?;
            *object.as_mut_state() = ctx;
            object.set_property("context_id", instance_id.unwrap_or(0) as i64);
            Ok::<_, phper::Error>(object)
        })
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(String::from(
            CONTEXT_INTERFACE_NAME,
        ))));

    // The `ArrayAccess` half of `ContextStorageScopeInterface`. The offsets are a scope's own
    // scratch space: nothing in this extension reads them, they exist so instrumentation can
    // use `$scope['key']` the way it can upstream.
    class
        .add_method("offsetExists", Visibility::Public, |this, arguments| {
            let key = offset_key(&arguments[0])?;
            let exists = local_storage(this)
                .as_z_arr()
                .is_some_and(|array| array.exists(&*key));
            Ok::<_, phper::Error>(exists)
        })
        .argument(Argument::new("offset").with_type_hint(ArgumentTypeHint::Mixed))
        .return_type(ReturnType::new(ReturnTypeHint::Bool));

    class
        .add_method("offsetGet", Visibility::Public, |this, arguments| {
            let key = offset_key(&arguments[0])?;
            // Null for an absent offset, matching `$array['missing'] ?? null` rather than
            // raising -- upstream's node does the same.
            let value = local_storage(this)
                .as_z_arr()
                .and_then(|array| array.get(&*key))
                .cloned()
                .unwrap_or_default();
            Ok::<_, phper::Error>(value)
        })
        .argument(Argument::new("offset").with_type_hint(ArgumentTypeHint::Mixed))
        .return_type(ReturnType::new(ReturnTypeHint::Mixed));

    class
        .add_method("offsetSet", Visibility::Public, |this, arguments| {
            let key = offset_key(&arguments[0])?;
            let value = arguments[1].clone();
            if let Some(array) = local_storage(this).as_mut_z_arr() {
                array.insert(&*key, value);
            }
            Ok::<_, phper::Error>(())
        })
        .argument(Argument::new("offset").with_type_hint(ArgumentTypeHint::Mixed))
        .argument(Argument::new("value").with_type_hint(ArgumentTypeHint::Mixed))
        .return_type(ReturnType::new(ReturnTypeHint::Void));

    class
        .add_method("offsetUnset", Visibility::Public, |this, arguments| {
            let key = offset_key(&arguments[0])?;
            if let Some(array) = local_storage(this).as_mut_z_arr() {
                array.remove(&*key);
            }
            Ok::<_, phper::Error>(())
        })
        .argument(Argument::new("offset").with_type_hint(ArgumentTypeHint::Mixed))
        .return_type(ReturnType::new(ReturnTypeHint::Void));
}
