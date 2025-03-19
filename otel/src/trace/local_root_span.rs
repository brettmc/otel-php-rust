use phper::{
    classes::{
        ClassEntity,
        Visibility,
    },
};
use crate::trace::span;
use crate::trace::span::SpanClass;
const LOCAL_ROOT_SPAN_CLASS_NAME: &str = r"OpenTelemetry\API\Trace\LocalRootSpan";

pub fn make_local_root_span_class(span_class: SpanClass) -> ClassEntity<()> {
    let mut class = ClassEntity::new(LOCAL_ROOT_SPAN_CLASS_NAME);

    class.add_static_method("current", Visibility::Public, move |_| {
        let object = span_class.init_object()?;
        let _ctx = span::get_local_root_span();
        Ok::<_, phper::Error>(object)
    });

    class
}