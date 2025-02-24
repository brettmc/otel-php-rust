use phper::{
    alloc::ToRefOwned,
    classes::{ClassEntity, StateClass, Visibility},
    functions::Argument,
};
use std::{
    borrow::Cow,
    convert::Infallible,
};
use opentelemetry::{
    Context,
    KeyValue,
    trace::{
        Span,
        SpanContext,
        Status,
        TraceContextExt,
    }
};
use opentelemetry_sdk::trace::Span as SdkSpan;
use crate::trace::span_context::SpanContextClass;
use crate::trace::current_span::CurrentSpanClass;
use crate::trace::scope::ScopeClass;

const SPAN_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\Span";

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
            if let Some(span) = this.as_mut_state().as_mut() {
                span.end();
            }
            Ok(())
        });

    class
        .add_method("setStatus", Visibility::Public, |this, arguments| {
            let span: &mut SdkSpan = this.as_mut_state().as_mut().unwrap();
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
            match status.as_str() {
                "Ok" => {
                    span.set_status(Status::Ok);
                }
                "Unset" => {
                    span.set_status(Status::Unset);
                }
                "Error" => {
                    span.set_status(Status::Error {
                        description,
                    });
                }
                _ => {

                }
            };
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::by_val("code"))
        .argument(Argument::by_val_optional("description"));

    class
        .add_method("setAttribute", Visibility::Public, |this, arguments| {
            let span: &mut SdkSpan = this.as_mut_state().as_mut().unwrap();
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();
            let value = arguments[1].expect_z_str()?.to_str()?.to_string();
            span.set_attribute(KeyValue::new(name, value));
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("setAttributes", Visibility::Public, |this, arguments| {
            let _span: &mut SdkSpan = this.as_mut_state().as_mut().unwrap();
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
            let span: &mut SdkSpan = this.as_mut_state().as_mut().unwrap();
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();
            span.update_name(name);
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("recordException", Visibility::Public, |this, _arguments| {
            let _span: &mut SdkSpan = this.as_mut_state().as_mut().unwrap();
            //TODO: implement
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addLink", Visibility::Public, |this, _arguments| {
            let _span: &mut SdkSpan = this.as_mut_state().as_mut().unwrap();
            //TODO: implement
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addEvent", Visibility::Public, |this, _arguments| {
            let _span: &mut SdkSpan = this.as_mut_state().as_mut().unwrap();
            //TODO: implement
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("getContext", Visibility::Public, move |this, _| {
            let span: &mut SdkSpan = this.as_mut_state().as_mut().unwrap();
            let span_context: SpanContext = span.span_context().clone();
            let mut object = span_context_class.init_object()?;
            *object.as_mut_state() = Some(span_context);
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
            let guard = ctx.attach();

            let mut object = scope_class.init_object()?;
            *object.as_mut_state() = Some(guard);
            Ok::<_, phper::Error>(object)
        });

    class
}
