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
        SpanContext,
        Status,
        TraceContextExt,
    }
};
use crate::trace::span_context::SpanContextClass;

/// Since you can't do much with a SpanRef, instead of returning a BoxedSpan from Span::getCurrent, we
/// just return this class (which should/will implement SpanInterface). It always operates on whatever
/// the "current span" is, rather than it being a "real" span.

const CURRENT_SPAN_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\CurrentSpan";
pub type CurrentSpanClass = StateClass<()>;

pub fn make_current_span_class(span_context_class: SpanContextClass) -> ClassEntity<()> {
    let mut class =
        ClassEntity::<()>::new_with_default_state_constructor(CURRENT_SPAN_CLASS_NAME);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("end", Visibility::Public, |_, _| -> phper::Result<()> {
            let ctx = Context::current();
            let span = ctx.span();
            span.end();
            Ok(())
        });

    class
        .add_method("setStatus", Visibility::Public, |this, arguments| {
            let ctx = Context::current();
            let span = ctx.span();
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
            let ctx = Context::current();
            let span = ctx.span();
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();
            let value = arguments[1].expect_z_str()?.to_str()?.to_string();
            span.set_attribute(KeyValue::new(name, value));
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("setAttributes", Visibility::Public, |this, arguments| {
            let ctx = Context::current();
            let _span = ctx.span();
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
            let ctx = Context::current();
            let span = ctx.span();
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();
            span.update_name(name);
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("recordException", Visibility::Public, |this, _arguments| {
            let ctx = Context::current();
            let _span = ctx.span();
            //TODO: implement record_error
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addLink", Visibility::Public, |this, _arguments| {
            let ctx = Context::current();
            let _span = ctx.span();
            //TODO: implement
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addEvent", Visibility::Public, |this, _arguments| {
            let ctx = Context::current();
            let _span = ctx.span();
            //TODO: implement add_event
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("getContext", Visibility::Public, move |_, _| {
            let ctx = Context::current();
            let span = ctx.span();
            let span_context: SpanContext = span.span_context().clone();
            let mut object = span_context_class.init_object()?;
            *object.as_mut_state() = Some(span_context);
            Ok::<_, phper::Error>(object)
        });

    class
}
