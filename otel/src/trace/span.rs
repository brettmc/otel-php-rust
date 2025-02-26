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
    convert::Infallible,
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
    current_span::CurrentSpanClass,
    scope::ScopeClass,
};

const SPAN_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\Span";

thread_local! {
    static CONTEXT_STORAGE: RefCell<Option<Context>> = RefCell::new(None);
}

pub type SpanClass = StateClass<Option<SdkSpan>>;

pub fn make_span_class(
    scope_class: ScopeClass,
    span_context_class: SpanContextClass,
    current_span_class: CurrentSpanClass,
) -> ClassEntity<Option<SdkSpan>> {
    let mut class =
        ClassEntity::<Option<SdkSpan>>::new_with_default_state_constructor(SPAN_CLASS_NAME);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("end", Visibility::Public, |this, _| -> phper::Result<()> {
            this.as_mut_state()
                .as_mut()
                .map(|span| span.end())
                .or_else(|| {
                    CONTEXT_STORAGE.with(|storage| {
                        storage.borrow().as_ref().map(|ctx| ctx.span().end())
                    })
                });
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

            this.as_mut_state()
                .as_mut()
                .map(|span| span.set_status(status_code.clone()))
                .or_else(|| {
                    CONTEXT_STORAGE.with(|storage| {
                        storage.borrow().as_ref().map(|ctx| ctx.span().set_status(status_code))
                    })
                });
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::by_val("code"))
        .argument(Argument::by_val_optional("description"));

    class
        .add_method("setAttribute", Visibility::Public, |this, arguments| {
            let key = arguments[0].expect_z_str()?.to_str()?.to_string();
            let value = &arguments[1];
            if let Some(kv) = zval_to_key_value(&key, &value) {
                this.as_mut_state()
                    .as_mut()
                    .map(|span| span.set_attribute(kv.clone()))
                    .or_else(|| {
                        CONTEXT_STORAGE.with(|storage| {
                            storage.borrow().as_ref().map(|ctx| ctx.span().set_attribute(kv))
                        })
                    });
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
            this.as_mut_state()
                .as_mut()
                .map(|span| span.set_attributes(result.clone()))
                .or_else(|| {
                    CONTEXT_STORAGE.with(|storage| {
                        storage.borrow().as_ref().map(|ctx| ctx.span().set_attributes(result.clone()))
                    })
                });

            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("updateName", Visibility::Public, |this, arguments| {
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();
            this.as_mut_state()
                .as_mut()
                .map(|span| span.update_name(name.clone()))
                .or_else(|| {
                    CONTEXT_STORAGE.with(|storage| {
                        storage.borrow().as_ref().map(|ctx| ctx.span().update_name(name))
                    })
                });

            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("recordException", Visibility::Public, |this, arguments| {
            let t: ZObject = arguments[0].expect_mut_z_obj()?.to_ref_owned();
            if let Ok(throwable) = ThrowObject::new(t) {
                this.as_mut_state()
                    .as_mut()
                    .map(|span| span.record_error(&throwable))
                    .or_else(|| {
                        CONTEXT_STORAGE.with(|storage| {
                            storage.borrow().as_ref().map(|ctx| ctx.span().record_error(&throwable))
                        })
                    });
            }
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addLink", Visibility::Public, |this, arguments| {
            let span_context_obj: &mut ZObj = arguments[0].expect_mut_z_obj()?;
            // TODO panics here: called `Option::unwrap()` on a `None` value
            let _span_context: &SpanContext = unsafe {
                span_context_obj.as_state_obj::<SpanContext>().as_state()
            };

            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addEvent", Visibility::Public, |this, _arguments| {
            //TODO: implement
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("getContext", Visibility::Public, move |this, _| {
            let span_context = this.as_state()
                .as_ref()
                .map(|span| span.span_context().clone())
                .or_else(|| {
                    CONTEXT_STORAGE.with(|storage| {
                        storage.borrow().as_ref().map(|ctx| ctx.span().span_context().clone())
                    })
                });
            let mut object = span_context_class.init_object()?;
            match span_context {
                Some(ctx) => {
                    *object.as_mut_state() = Some(ctx);
                }
                None => {}, //TODO this shouldn't happen!
            }
            Ok::<_, phper::Error>(object)
        });

    class
        .add_static_method("getCurrent", Visibility::Public, move |_| {
            let object = current_span_class.init_object()?;
            Ok::<_, phper::Error>(object)
        });

    class
        .add_method("activate", Visibility::Public, move |this, _arguments| {
            let span = this.as_mut_state().take().expect("No span stored!");
            let ctx = Context::current_with_span(span);
            CONTEXT_STORAGE.with(|storage| *storage.borrow_mut() = Some(ctx.clone()));

            let guard = ctx.attach();

            let mut object = scope_class.init_object()?;
            *object.as_mut_state() = Some(guard);
            Ok::<_, phper::Error>(object)
        });

    class
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