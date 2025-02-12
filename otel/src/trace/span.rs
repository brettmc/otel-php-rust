use phper::{
    alloc::ToRefOwned,
    classes::{ClassEntity, StaticStateClass, Visibility},
    functions::Argument,
};
use std::{
    convert::Infallible,
};
use opentelemetry::trace::{
    Span,
    Status,
};
use opentelemetry::global::{
    BoxedSpan,
};

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
        .add_method("setStatus", Visibility::Public, |this, _arguments| {
            let span: &mut BoxedSpan = this.as_mut_state().as_mut().unwrap();
            span.set_status(Status::Ok);
            Ok::<_, phper::Error>(this.to_ref_owned())
        })
        .argument(Argument::by_val("code"))
        .argument(Argument::by_val_optional("description"));

    class
}
