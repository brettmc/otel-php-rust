use phper::{
    sys,
    values::{
        ExecuteData,
    },
    strings::ZStr,
};
use opentelemetry::{
    ContextGuard,
    global,
    Context,
    trace::{
        Tracer,
        TraceContextExt,
    }
};
use std::{
    collections::HashMap,
    cell::RefCell,
};

thread_local! {
    static CONTEXT_GUARD_MAP: RefCell<HashMap<usize, ContextGuard>> = RefCell::new(HashMap::new());
}

fn store_guard(exec_ptr: *mut sys::zend_execute_data, guard: ContextGuard) {
    let key = exec_ptr as usize;
    CONTEXT_GUARD_MAP.with(|map| {
        map.borrow_mut().insert(key, guard);
    });
}

fn take_guard(exec_ptr: *mut sys::zend_execute_data) -> Option<ContextGuard> {
    let key = exec_ptr as usize;
    CONTEXT_GUARD_MAP.with(|map| map.borrow_mut().remove(&key))
}

pub unsafe extern "C" fn observer_begin(execute_data: *mut sys::zend_execute_data) {
    if let Some(exec_data) = ExecuteData::try_from_mut_ptr(execute_data) {
        let (function_name, class_name) = match get_function_and_class_name(exec_data) {
            Ok(names) => names,
            Err(_) => (None, None),
        };
        let span_name = format!(
            "{}::{}",
            class_name.as_deref().unwrap_or("<global>"),
            function_name.as_deref().unwrap_or("<anonymous>")
        );
        let tracer = global::tracer("php-auto-instrumentation");
        let span = tracer.start(span_name);
        let ctx = Context::current_with_span(span);
        let guard = ctx.attach();
        store_guard(execute_data, guard);
    }
}

pub unsafe extern "C" fn observer_end(
    execute_data: *mut sys::zend_execute_data,
    _return_value: *mut sys::zval,
) {
    if let Some(guard) = take_guard(execute_data) {
        // Dropping the guard detaches the context and finishes the span.
        drop(guard);
    } else {
        println!("No active opentelemetry span guard found for execute_data at: {:p}", execute_data);
    }
}

pub unsafe extern "C" fn observer_instrument(execute_data: *mut sys::zend_execute_data) -> sys::zend_observer_fcall_handlers {
    if let Some(exec_data) = ExecuteData::try_from_mut_ptr(execute_data) {
        let (function_name, class_name) = match get_function_and_class_name(exec_data) {
            Ok(names) => names,
            Err(_) => (None, None),
        };
        if should_trace(class_name.as_deref(), function_name.as_deref()) {
            return sys::zend_observer_fcall_handlers {
                begin: Some(observer_begin),
                end: Some(observer_end),
            };
        }
    }
    sys::zend_observer_fcall_handlers {
        begin: None,
        end: None,
    }
}

//coped from https://github.com/apache/skywalking-php/blob/v0.8.0/src/execute.rs#L283
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

//TODO implement plugins for various applications/PSRs/etc, and query
//     each of them for interest in this function/method
fn should_trace(class_name: Option<&str>, function_name: Option<&str>) -> bool {
    match (class_name, function_name) {
        (Some("DemoClass"), Some(_)) => true,
        (None, Some("demoFunction")) => true,
        (Some("DemoClass"), None) => true,
        (None, Some("str_contains")) => true,
        _ => false,
    }
}