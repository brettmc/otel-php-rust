use phper::{
    alloc::ToRefOwned,
    classes::{ClassEntity, StateClass, Visibility},
    functions::Argument,
};
use std::{
    borrow::Cow,
    cell::RefCell,
    convert::Infallible,
};
use opentelemetry::{
    Context,
    KeyValue,
    trace::{
        Span,
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
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();
            let value = arguments[1].expect_z_str()?.to_str()?.to_string();
            let attribute = KeyValue::new(name, value);
            this.as_mut_state()
                .as_mut()
                .map(|span| span.set_attribute(attribute.clone()))
                .or_else(|| {
                    CONTEXT_STORAGE.with(|storage| {
                        storage.borrow().as_ref().map(|ctx| ctx.span().set_attribute(attribute))
                    })
                });

            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("setAttributes", Visibility::Public, |this, arguments| {
            let attributes = arguments[0].expect_z_arr()?;
            for (_key, _value) in attributes.iter() {
                //TODO: iterate over attributes, apply to span
                //echo!("{:?}:{:?}\n", key, value);
                //span.set_attribute(KeyValue::new(key_str, value_str));
            }
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
        .add_method("recordException", Visibility::Public, |this, _arguments| {
            //TODO: implement
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addLink", Visibility::Public, |this, _arguments| {
            //TODO: implement
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
