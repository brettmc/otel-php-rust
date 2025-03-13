use phper::{
    arrays::IterKey,
    alloc::ToRefOwned,
    classes::{ClassEntity, StateClass, Visibility},
    errors::ThrowObject,
    functions::Argument,
    objects::{ZObj, ZObject},
    values::ZVal,
};
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    convert::Infallible,
    sync::atomic::{AtomicU64, Ordering},
};
use opentelemetry::{
    Array,
    Context,
    KeyValue,
    Value,
    StringValue,
    trace::{
        Span,
        SpanContext,
        Status,
        TraceContextExt,
    }
};
use opentelemetry_sdk::trace::Span as SdkSpan;
use crate::trace::{
    span_context::SpanContextClass,
    scope::ScopeClass,
};

const SPAN_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\Span";

// The span related to a class instance is either stored as a class entity (SdkSpan) if the span has been
// started but not activated. Once it has been activated, the class entity is set to None, and the context
// is stored in CONTEXT_STORAGE, and a reference to the context created and stored as a class property.
// Each method that operates on the span needs to check for SdkSpan then stored context, and then operate on
// either the SdkSpan or context.span()
thread_local! {
    static CONTEXT_STORAGE: RefCell<HashMap<u64, Context>> = RefCell::new(HashMap::new());
}
static INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

pub type SpanClass = StateClass<Option<SdkSpan>>;

pub fn make_span_class(
    scope_class: ScopeClass,
    span_context_class: SpanContextClass,
) -> ClassEntity<Option<SdkSpan>> {
    let mut class =
        ClassEntity::<Option<SdkSpan>>::new_with_default_state_constructor(SPAN_CLASS_NAME);
    let span_class = class.bind_class();

    class.add_property("context_id", Visibility::Private, 0i64);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("end", Visibility::Public, |this, _| -> phper::Result<()> {
            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            if let Some(span) = this.as_mut_state().as_mut() {
                span.end();
            } else if let Some(ctx) = get_context_instance(instance_id as u64) {
                ctx.span().end();
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
            } else if let Some(ctx) = get_context_instance(instance_id as u64) {
                ctx.span().set_status(status_code);
            }

            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::by_val("code"))
        .argument(Argument::by_val_optional("description"));

    class
        .add_method("setAttribute", Visibility::Public, |this, arguments| {
            let key = arguments[0].expect_z_str()?.to_str()?.to_string();
            let value = &arguments[1];
            if let Some(kv) = zval_to_key_value(&key, &value) {
                let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
                if let Some(span) = this.as_mut_state().as_mut() {
                    span.set_attribute(kv.clone());
                } else if let Some(ctx) = get_context_instance(instance_id as u64) {
                    ctx.span().set_attribute(kv.clone());
                }
             }

            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("setAttributes", Visibility::Public, |this, arguments| {
            let attributes = arguments[0].expect_z_arr()?;
            let mut result = Vec::new();
            for (key, value) in attributes.iter() {
                match key {
                    IterKey::Index(_) => {}, // Skip integer keys
                    IterKey::ZStr(zstr) => {
                        if let Some(key_str) = zstr.to_str().ok().map(|s| s.to_string()) {
                            if let Some(kv) = zval_to_key_value(&key_str, value) {
                                result.push(kv);
                            }
                        }
                    },
                };
            }

            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            if let Some(span) = this.as_mut_state().as_mut() {
                span.set_attributes(result.clone());
            } else if let Some(ctx) = get_context_instance(instance_id as u64) {
                ctx.span().set_attributes(result.clone());
            }

            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("updateName", Visibility::Public, |this, arguments| {
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();
            // this.as_mut_state()
            //     .as_mut()
            //     .map(|span| span.update_name(name.clone()))
            //     .or_else(|| {
            //         CONTEXT_STORAGE.with(|storage| {
            //             storage.borrow().as_ref().map(|ctx| ctx.span().update_name(name))
            //         })
            //     });

            let instance_id = this.get_property("context_id").as_long().unwrap_or(0);
            if let Some(span) = this.as_mut_state().as_mut() {
                span.update_name(name.clone());
            } else if let Some(ctx) = get_context_instance(instance_id as u64) {
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
                } else if let Some(ctx) = get_context_instance(instance_id as u64) {
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
            } else if let Some(_ctx) = get_context_instance(instance_id as u64) {
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
                                if let Some(kv) = zval_to_key_value(&key_str, value) {
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
            } else if let Some(ctx) = get_context_instance(instance_id as u64) {
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
            } else if let Some(ctx) = get_context_instance(instance_id as u64) {
                *object.as_mut_state() = Some(ctx.span().span_context().clone());
            }
            Ok::<_, phper::Error>(object)
        });

    class
        .add_static_method("getCurrent", Visibility::Public, move |_| {
            let ctx = Context::current();
            //let instance_id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed); // Generate a unique ID for this instance
            let instance_id = new_instance_id();
            CONTEXT_STORAGE.with(|storage| {
                storage.borrow_mut().insert(instance_id, ctx.clone());
            });
            let mut object = span_class.clone().init_object()?;
            *object.as_mut_state() = None;
            object.set_property("context_id", instance_id as i64);

            Ok::<_, phper::Error>(object)
        });

    class
        .add_method("activate", Visibility::Public, move |this, _arguments| {
            let span = this.as_mut_state().take().expect("No span stored!");
            let ctx = Context::current_with_span(span);
            let instance_id = new_instance_id();
            CONTEXT_STORAGE.with(|storage| storage.borrow_mut().insert(instance_id, ctx.clone()));
            this.set_property("context_id", instance_id as i64);

            let guard = ctx.attach();

            let mut object = scope_class.init_object()?;
            *object.as_mut_state() = Some(guard);
            Ok::<_, phper::Error>(object)
        });

    class
}

pub fn get_context_instance(instance_id: u64) -> Option<Context> {
    if instance_id == 0 {
        None
    } else {
        CONTEXT_STORAGE.with(|storage| storage.borrow().get(&instance_id).cloned())
    }
}

pub fn new_instance_id() -> u64 {
    INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn zval_to_key_value(key: &str, value: &ZVal) -> Option<KeyValue> {
    let type_info = value.get_type_info();
    if type_info.is_string() {
        return value.as_z_str().and_then(|z| z.to_str().ok()).map(|s| KeyValue::new(key.to_string(), s.to_string()));
    }
    if type_info.is_long() {
        return value.as_long().map(|v| KeyValue::new(key.to_string(), v));
    }
    if type_info.is_double() {
        return value.as_double().map(|v| KeyValue::new(key.to_string(), v));
    }
    if type_info.is_bool() {
        return value.as_bool().map(|v| KeyValue::new(key.to_string(), v));
    }
    if type_info.is_array() {
        return zval_to_vec(key, value);
    }
    None
}

fn zval_to_vec(key: &str, value: &ZVal) -> Option<KeyValue> {
    let array = value.as_z_arr()?;

    let mut string_values = Vec::new();
    let mut int_values = Vec::new();
    let mut float_values = Vec::new();
    let mut bool_values = Vec::new();

    for (_, v) in array.iter() {
        if let Some(val) = v.as_z_str().and_then(|z| z.to_str().ok()) {
            string_values.push(val.to_string());
        } else if let Some(val) = v.as_long() {
            int_values.push(val);
        } else if let Some(val) = v.as_double() {
            float_values.push(val);
        } else if let Some(val) = v.as_bool() {
            bool_values.push(val);
        }
    }

    if !string_values.is_empty() {
        return Some(KeyValue::new(
            key.to_string(),
            Value::Array(Array::from(
                string_values.into_iter().map(StringValue::from).collect::<Vec<_>>(),
            )),
        ));
    } else if !int_values.is_empty() {
        return Some(KeyValue::new(
            key.to_string(),
            Value::Array(Array::from(int_values)),
        ));
    } else if !float_values.is_empty() {
        return Some(KeyValue::new(
            key.to_string(),
            Value::Array(Array::from(float_values)),
        ));
    } else if !bool_values.is_empty() {
        return Some(KeyValue::new(
            key.to_string(),
            Value::Array(Array::from(bool_values)),
        ));
    }

    None
}