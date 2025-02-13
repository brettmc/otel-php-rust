use phper::{
    alloc::ToRefOwned,
    classes::{ClassEntity, StaticStateClass, Visibility},
    functions::Argument,
};
use std::{
    borrow::Cow,
    convert::Infallible,
};
use opentelemetry::{
    Context,
    KeyValue,
    global::BoxedSpan,
    trace::{
        Span,
        SpanContext,
        Status,
        TraceContextExt,
    }
};
use crate::trace::span_context::SPAN_CONTEXT_CLASS;

const SPAN_CLASS_NAME: &str = "OpenTelemetry\\API\\Trace\\Span";

pub static SPAN_CLASS: StaticStateClass<Option<BoxedSpan>> = StaticStateClass::null();

pub fn make_span_class() -> ClassEntity<Option<BoxedSpan>> {
    let mut class =
        ClassEntity::<Option<BoxedSpan>>::new_with_default_state_constructor(SPAN_CLASS_NAME);

    class.bind(&SPAN_CLASS);

    class.add_method("__construct", Visibility::Private, |_, _| {
        Ok::<_, Infallible>(())
    });

    class
        .add_method("end", Visibility::Public, |this, _| -> phper::Result<()> {
            let span: &mut BoxedSpan = this.as_mut_state().as_mut().unwrap();
            span.end();
            Ok(())
        });

    class
        .add_method("setStatus", Visibility::Public, |this, arguments| {
            let span: &mut BoxedSpan = this.as_mut_state().as_mut().unwrap();
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
            let span: &mut BoxedSpan = this.as_mut_state().as_mut().unwrap();
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();
            let value = arguments[1].expect_z_str()?.to_str()?.to_string();
            span.set_attribute(KeyValue::new(name, value));
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("setAttributes", Visibility::Public, |this, arguments| {
            let _span: &mut BoxedSpan = this.as_mut_state().as_mut().unwrap();
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
            let span: &mut BoxedSpan = this.as_mut_state().as_mut().unwrap();
            let name = arguments[0].expect_z_str()?.to_str()?.to_string();
            span.update_name(name);
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("recordException", Visibility::Public, |this, _arguments| {
            let _span: &mut BoxedSpan = this.as_mut_state().as_mut().unwrap();
            //TODO: implement
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addLink", Visibility::Public, |this, _arguments| {
            let _span: &mut BoxedSpan = this.as_mut_state().as_mut().unwrap();
            //TODO: implement
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("addEvent", Visibility::Public, |this, _arguments| {
            let _span: &mut BoxedSpan = this.as_mut_state().as_mut().unwrap();
            //TODO: implement
            Ok::<_, phper::Error>(this.to_ref_owned())
        });

    class
        .add_method("getContext", Visibility::Public, |this, _| {
            let span: &mut BoxedSpan = this.as_mut_state().as_mut().unwrap();
            let span_context: SpanContext = span.span_context().clone();
            let mut object = SPAN_CONTEXT_CLASS.init_object()?;
            *object.as_mut_state() = Some(span_context);
            Ok::<_, phper::Error>(object)
        });

    class
        .add_static_method("getCurrent", Visibility::Public, |_| {
            let ctx = Context::current();
            let _span = ctx.span();
            let mut _object = SPAN_CLASS.init_object()?;
            //*object.as_mut_state() = Some(span);
            Ok::<_, phper::Error>(_object)
        });

    class
        .add_method("activate", Visibility::Public, |_this, _arguments| -> phper::Result<()> {
            //TODO: activate span, wrap `guard` in a Scope than can be `detached()`ed
            // let span: BoxedSpan = this.as_mut_state().as_ref().unwrap().clone();
            // let ctx = Context::current().with_span(span); //@see https://docs.rs/opentelemetry/latest/opentelemetry/trace/trait.TraceContextExt.html#examples-1
            // let _guard = ctx.attach();
            Ok(())
        });

    class
}
