use phper::{
    classes::{ClassEntity, Interface, StateClass, Visibility},
    functions::{Argument, ReturnType},
    types::{ArgumentTypeHint, ReturnTypeHint},
    values::ZVal,
    objects::ZObj,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::convert::Infallible;
use crate::context::{
    context::ContextClassEntity,
    storage,

};

pub type TraceContextPropagatorClass = StateClass<()>;

pub fn make_trace_context_propagator_class(
    text_map_propagator_interface: Interface,
    context_class: &ContextClassEntity,
) -> ClassEntity<()> {
    let mut class = ClassEntity::<()>::new_with_default_state_constructor(
        r"OpenTelemetry\API\Propagation\TraceContextPropagator",
    );

    class.implements(text_map_propagator_interface);

    class.add_method("__construct", Visibility::Private, |this, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("inject", Visibility::Public, |this, arguments| -> phper::Result<()> {
            // Context (optional, default to Context::current)
            /*let context_obj = arguments[2].expect_mut_z_obj()?;
            let context_id = context_obj.get_property("context_id").as_long().unwrap_or(0);

            let context = storage::get_context_instance(context_id as u64);

            // Carrier (PHP array passed by ref)
            //if let Some(array) = arguments.get(1).and_then(|attrs| attrs.as_z_arr()) {
            let carrier = arguments[0].as_z_arr()?;
            let mut out_map = std::collections::HashMap::<String, String>::new();

            // Use global propagator to inject
            opentelemetry::global::get_text_map_propagator(|prop| {
                prop.inject_context(&context, &mut |k, v| {
                    out_map.insert(k.to_string(), v.to_string());
                });
            });

            for (k, v) in out_map {
                carrier.insert(ZVal::from(k), ZVal::from(v));
            }

            Ok::<_, phper::Error>(())*/
            Ok(())
        })
        .argument(Argument::new("carrier").with_type_hint(ArgumentTypeHint::Mixed).by_ref())
        .argument(Argument::new("setter").allow_null().with_default_value("null"))
        .argument(Argument::new("context")
            .with_type_hint(ArgumentTypeHint::ClassEntry(r"OpenTelemetry\Context\ContextInterface".to_string()))
            .with_default_value("null")
            .allow_null()
        )
        .return_type(ReturnType::new(ReturnTypeHint::Void));

    let context_ce = context_class.bound_class();
    class
        .add_method("extract", Visibility::Public, move |this, arguments| {
            // Carrier (input headers)
            let carrier = arguments[0].expect_z_arr()?;

            let mut map = std::collections::HashMap::<String, String>::new();
            for (k, v) in carrier.iter() {
                if let phper::arrays::IterKey::ZStr(k) = k {
                    if let Some(zstr) = v.as_z_str() {
                        if let Ok(val) = zstr.to_str() {
                            map.insert(k.to_str()?.to_string(), val.to_string());
                        }
                    }
                }
            }

            // Parent context (optional)
            let context_id = match arguments.get(2) {
                Some(val) => {
                    let obj = val.expect_z_obj()?;
                    let id = obj.get_property("context_id").as_long().unwrap_or(0);
                    if id > 0 { Some(id as u64) } else { None }
                }
                None => None,
            };

            let parent_cx = context_id
                .and_then(|id| storage::get_context_instance(id))
                .unwrap_or_else(|| Arc::new(opentelemetry::Context::current()));

            // Extract new context from headers
            let new_cx = opentelemetry::global::get_text_map_propagator(|prop| {
                prop.extract_with_context(&parent_cx, &map)
            });
            let instance_id = storage::store_context_instance(Arc::new(new_cx));

            // Wrap in PHP context object
            let mut obj = context_ce.init_object()?;
            obj.set_property("context_id", instance_id as i64);
            Ok::<_, phper::Error>(obj)
        })
        .argument(Argument::new("carrier"))
        .argument(Argument::new("getter").allow_null().with_default_value("null"))
        .argument(Argument::new("context")
            .with_type_hint(ArgumentTypeHint::ClassEntry(r"OpenTelemetry\Context\ContextInterface".to_string()))
            .with_default_value("null")
            .allow_null()
        )
        .return_type(ReturnType::new(ReturnTypeHint::ClassEntry(
            r"OpenTelemetry\Context\ContextInterface".to_string()
        )));

    class
}
