use phper::{
    arrays::IterKey,
    alloc::ToRefOwned,
    classes::{ClassEntity, Interface, StateClass, Visibility},
    errors::ThrowObject,
    functions::Argument,
    objects::{ZObj, ZObject},
};
use std::{
    borrow::Cow,
    convert::Infallible,
    sync::Arc,
};
use opentelemetry::{
    Context,
    trace::{
        Span,
        SpanContext,
        Status,
        TraceContextExt,
    }
};
use opentelemetry_sdk::trace::Span as SdkSpan;
use crate::{
    context::{
        context::ContextClass,
        scope::ScopeClass,
        storage,
    },
    trace::{
        span_context::SpanContextClass,
        local_root_span::store_local_root_span,
    },
    util,
};

const SPAN_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\Span";

pub type SpanClass = StateClass<Option<SdkSpan>>;

pub fn make_span_class(
    scope_class: ScopeClass,
    span_context_class: SpanContextClass,
    context_class: ContextClass,
    span_interface: &Interface,
) -> ClassEntity<Option<SdkSpan>> {
    let mut class =
        ClassEntity::<Option<SdkSpan>>::new_with_default_state_constructor(SPAN_CLASS_NAME);
    let span_class = class.bound_class();

    class.implements(span_interface.clone());

    class.add_property("context_id", Visibility::Private, 0i64);
    class.add_property("is_local_root", Visibility::Private, false);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("end", Visibility::Public, |this, _| -> phper::Result<()> {
            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            if let Some(span) = this.as_mut_state().as_mut() {
                tracing::debug!("Span::Ending Span (SdkSpan)");
                span.end();
            } else {
                {
                    //in own block to ensure reference dropped before remove
                    if let Some(ctx) = storage::get_context_instance(instance_id as u64) {
                        tracing::debug!("Span::Ending Span (SpanRef)");
                        ctx.span().end();
                    }
                }
                storage::maybe_remove_context_instance(instance_id as u64);
            }

            Ok(())
        });

    class
        .add_method("setStatus", Visibility::Public, |this, arguments| {
            let status = match arguments[0].expect_z_str()?.to_str() {
                Ok(s) => s.to_string(),
                Err(_) => return Ok(this.to_ref_owned()), // Ignore invalid UTF-8 input
            };
            let description: Cow<'static, str> = arguments
                .get(1)
                .map(|d| d.expect_z_str())
                .transpose()?
                .map(|d| {
                    match d.to_str() {
                        Ok(s) => Cow::Owned(s.to_string()),
                        Err(_) => Cow::Borrowed(""),
                    }
                })
                .unwrap_or(Cow::Borrowed(""));
            let status_code = match status.as_str() {
                "Ok" => Status::Ok,
                "Unset" => Status::Unset,
                "Error" => Status::Error {description},
                _ => Status::Unset,
            };

            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            if let Some(span) = this.as_mut_state().as_mut() {
                span.set_status(status_code.clone());
            } else if let Some(ctx) = storage::get_context_instance(instance_id as u64) {
                ctx.span().set_status(status_code);
            }

            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::new("code"))
        .argument(Argument::new("description").optional());

    class
        .add_method("setAttribute", Visibility::Public, |this, arguments| {
            let key = arguments[0].expect_z_str()?.to_str()?.to_string();
            let value = &arguments[1];
            if let Some(kv) = util::zval_to_key_value(&key, &value) {
                let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
                if let Some(span) = this.as_mut_state().as_mut() {
                    span.set_attribute(kv.clone());
                } else if let Some(ctx) = storage::get_context_instance(instance_id as u64) {
                    ctx.span().set_attribute(kv.clone());
                }
             }

            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("setAttributes", Visibility::Public, |this, arguments| {
            let attrs = arguments[0]
                .expect_z_arr()?
                .to_owned();

            let attributes = util::zval_arr_to_key_value_vec(attrs);

            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            if let Some(span) = this.as_mut_state().as_mut() {
                span.set_attributes(attributes);
            } else if let Some(ctx) = storage::get_context_instance(instance_id as u64) {
                ctx.span().set_attributes(attributes);
            }

            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("updateName", Visibility::Public, |this, arguments| {
            tracing::debug!("Span::updateName");
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();

            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            if let Some(span) = this.as_mut_state().as_mut() {
                tracing::debug!("Span::updateName (SdkSpan)");
                span.update_name(name.clone());
            } else if let Some(ctx) = storage::get_context_instance(instance_id as u64) {
                tracing::debug!("Span::updateName (SpanRef)");
                ctx.span().update_name(name);
            }
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("recordException", Visibility::Public, |this, arguments| {
            let t: ZObject = arguments[0].expect_mut_z_obj()?.to_ref_owned();
            if let Ok(throwable) = ThrowObject::new(t) {
                let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
                if let Some(span) = this.as_mut_state().as_mut() {
                    span.record_error(&throwable);
                } else if let Some(ctx) = storage::get_context_instance(instance_id as u64) {
                    ctx.span().record_error(&throwable);
                }
            }
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addLink", Visibility::Public, |this, arguments| {
            let span_context_obj: &mut ZObj = arguments[0].expect_mut_z_obj()?;
            let state_obj = unsafe { span_context_obj.as_state_obj::<Option<SpanContext>>() };
            let span_context = match state_obj.as_state() {
                Some(v) => v.clone(),
                None => return Err(phper::Error::boxed("Invalid SpanContext object")),
            };

            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            let attributes = vec![];
            if let Some(span) = this.as_mut_state().as_mut() {
                span.add_link(span_context.clone(), attributes);
            } else if let Some(_ctx) = storage::get_context_instance(instance_id as u64) {
                //SpanRef.add_link does not exist, so do nothing (see unreleased https://github.com/open-telemetry/opentelemetry-rust/pull/1515 )
                //ctx.span().add_link(&span_context.clone(), attributes);
            }

            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addEvent", Visibility::Public, |this, arguments| {
            let event_name = arguments[0].expect_z_str()?.to_str()?.to_string();
            let mut attributes = Vec::new();
            if let Some(array) = arguments.get(1).and_then(|attrs| attrs.as_z_arr()) {
                for (key, value) in array.iter() {
                    match key {
                        IterKey::Index(_) => {}, // Skip integer keys
                        IterKey::ZStr(zstr) => {
                            if let Some(key_str) = zstr.to_str().ok().map(|s| s.to_string()) {
                                if let Some(kv) = util::zval_to_key_value(&key_str, value) {
                                    attributes.push(kv);
                                }
                            }
                        },
                    };
                }
            }

            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            if let Some(span) = this.as_mut_state().as_mut() {
                span.add_event(event_name.clone(), attributes.clone());
            } else if let Some(ctx) = storage::get_context_instance(instance_id as u64) {
                ctx.span().add_event(event_name.clone(), attributes.clone());
            }

            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("getContext", Visibility::Public, move |this, _| {
            let mut object = span_context_class.init_object()?;
            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            if let Some(sdk_span) = this.as_state().as_ref() {
                *object.as_mut_state() = Some(sdk_span.span_context().clone());
            } else if let Some(ctx) = storage::get_context_instance(instance_id as u64) {
                *object.as_mut_state() = Some(ctx.span().span_context().clone());
            }
            Ok::<_, phper::Error>(object)
        });

    class
        .add_static_method("getCurrent", Visibility::Public, {
            let span_class = span_class.clone();
            move |_| {
                let ctx = storage::current_context(); // <- from storage.rs
                let instance_id = storage::store_context_instance(ctx.clone());

                let mut object = span_class.init_object()?;
                *object.as_mut_state() = None;
                object.set_property("context_id", instance_id as i64);

                Ok::<_, phper::Error>(object)
            }
        });

    //TODO should activate() use storeInContext()
    class
        .add_method("activate", Visibility::Public, {
            let scope_ce = scope_class.clone();
            move |this, _arguments| {
                let span = this.as_mut_state().take().expect("No span stored!");
                let is_local_root = !storage::current_context().span().span_context().is_valid();

                let ctx = Context::current_with_span(span);
                let instance_id = storage::store_context_instance(Arc::new(ctx.clone()));
                this.set_property("context_id", instance_id as i64);
                if is_local_root {
                    this.set_property("is_local_root", true);
                    store_local_root_span(instance_id);
                }

                storage::attach_context(instance_id).map_err(phper::Error::boxed)?;

                let mut object = scope_ce.init_object()?;
                object.set_property("context_id", instance_id as i64);
                Ok::<_, phper::Error>(object)
            }
        });

    class
        .add_method("storeInContext", Visibility::Public, move |this, arguments| {
            let span = this.as_mut_state().take().expect("No span stored!");

            let context_obj: &mut ZObj = arguments[0].expect_mut_z_obj()?;
            let context_id = context_obj.get_property("context_id").as_long().unwrap_or(0);
            // Always go through storage — handles context_id = 0 as Context::current()
            let context = storage::resolve_context(context_id as u64);
            let arc_ctx = Arc::new(context.with_span(span));
            let instance_id = storage::store_context_instance(arc_ctx.clone());

            let mut object = context_class.init_object()?;
            *object.as_mut_state() = Some(arc_ctx);
            object.set_property("context_id", instance_id as i64);

            Ok::<_, phper::Error>(object)
        }); //argument ContextInterface, return ContextInterface

    let span_class_clone = class.bound_class();
    class
        .add_static_method("fromContext", Visibility::Public, move |arguments| {
            //todo could this become a macro?? better, a generic macro?
            let context_obj: &mut ZObj = arguments[0].expect_mut_z_obj()?;
            let instance_id = context_obj.get_property("context_id").as_long().unwrap_or(0);
            let mut object = span_class_clone.init_object()?;
            *object.as_mut_state() = None;
            object.set_property("context_id", instance_id as i64);
            Ok::<_, phper::Error>(object)
        });

    class
}
