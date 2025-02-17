use phper::{
    strings::ZStr,
    values::{ExecuteData},
    sys::zend_observer_fcall_handlers,
    sys::zend_execute_data,
};
use opentelemetry::{
    global,
    global::BoxedSpan,
    Context,
    trace::{
        Span,
        Tracer,
        TraceContextExt,
    },
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

static mut ACTIVE_SPANS: Option<Mutex<HashMap<usize, BoxedSpan>>> = None;

pub unsafe extern "C" fn on_function_begin(execute_data: *mut zend_execute_data) {
    println!("on_function_begin");
    if let Some(execute_data) = ExecuteData::try_from_mut_ptr(execute_data) {
        if let Ok((Some(function), Some(class))) = get_function_and_class_name(execute_data) {
            if class == "DemoClass" && function == "test" {
                let tracer = global::tracer("php-auto-instrumentation");
                let span: BoxedSpan = tracer.start("DemoClass::test");

                // let active_spans = ACTIVE_SPANS.get_or_insert_with(|| Mutex::new(HashMap::new()));
                // active_spans.lock().unwrap().insert(execute_data as *const _ as usize, span.clone());

                // ✅ Use a reference to the stored span for activation
                let ctx = Context::current_with_span(span);
                let _guard = ctx.attach();
            }
        }
    }
}

/// ✅ Function observer hook (end)
pub unsafe extern "C" fn on_function_end(execute_data: *mut zend_execute_data, _retval: *mut phper::sys::zval) {
    println!("on_function_end");
    // if let Some(active_spans) = ACTIVE_SPANS.as_mut() {
    //     if let Some(span) = active_spans.lock().unwrap().remove(&(execute_data as *const _ as usize)) {
    //         span.end(); // ✅ End the span when function exits
    //     }
    // }

    ()
}

pub unsafe extern "C" fn on_function_call(execute_data: *mut zend_execute_data) -> zend_observer_fcall_handlers {
    /*if let Some(execute_data) = ExecuteData::try_from_mut_ptr(execute_data) {
        match get_function_and_class_name(execute_data) {
            Ok((Some(function), Some(class))) if class == "DemoClass" && function == "test" => {
                println!("✅ Entered {}::{}", class, function);
            }
            Ok((Some(function), None)) if function == "test" => {
                println!("✅ Entered function {}", function);
            }
            _ => {}
        }
    }*/

    // zend_observer_fcall_handlers { begin: None, end: None }

    zend_observer_fcall_handlers {
        begin: Some(on_function_begin),
        end: Some(on_function_end),
    }
}

/// ✅ Extracts the class and function name from `zend_execute_data`
fn get_function_and_class_name(
    execute_data: &mut ExecuteData,
) -> anyhow::Result<(Option<String>, Option<String>)> {
    let function = execute_data.func();

    let function_name = function
        .get_function_name()
        .map(ZStr::to_str)
        .transpose()?
        .map(ToOwned::to_owned);
    let class_name = function
        .get_class()
        .map(|cls| cls.get_name().to_str().map(ToOwned::to_owned))
        .transpose()?;

    Ok((function_name, class_name))
}
