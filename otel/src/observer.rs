// copied from https://github.com/skpr/compass/blob/v1.2.0/extension/src/execute.rs
use chrono::prelude::*;
use phper::{
    sys,
    values::ExecuteData,
    strings::ZStr,
};
use std::ptr::null_mut;
use opentelemetry::{
    global,
    Context,
    trace::{
        Tracer,
        TraceContextExt,
    }
};

static mut UPSTREAM_EXECUTE_EX: Option<
    unsafe extern "C" fn(execute_data: *mut sys::zend_execute_data),
> = None;
static mut UPSTREAM_EXECUTE_INTERNAL: Option<unsafe extern "C" fn(*mut sys::zend_execute_data, *mut sys::zval)> = None;

// This function swaps out the PHP exec function for our own. Allowing us to wrap it.
pub fn register_exec_functions() {
    unsafe {
        UPSTREAM_EXECUTE_EX = sys::zend_execute_ex;
        sys::zend_execute_ex = Some(execute_ex);

        //TODO: sys::zend_execute_internal seems not set at MINIT...
        // UPSTREAM_EXECUTE_INTERNAL = sys::zend_execute_internal;
        // sys::zend_execute_internal = Some(execute_internal);
    }
}

unsafe extern "C" fn execute_internal(execute_data: *mut sys::zend_execute_data, return_value: *mut sys::zval) {
    // println!("execute_internal");
    let execute_data = match ExecuteData::try_from_mut_ptr(execute_data) {
        Some(execute_data) => execute_data,
        None => {
            // println!("execute_internal::None");
            upstream_execute_internal(None, Some(return_value));
            return;
        }
    };

    let (function_name, class_name) = match get_function_and_class_name(execute_data) {
        Ok(names) => names,
        Err(_) => (None, None), // Handle errors gracefully
    };

    if should_trace(class_name.as_deref(), function_name.as_deref()) {
        let tracer = global::tracer("php-auto-instrumentation");
        let span_name = format!(
            "{}::{}",
            class_name.as_deref().unwrap_or("<global>"),
            function_name.as_deref().unwrap_or("<anonymous>")
        );
        let span = tracer.start(span_name);
        let ctx = Context::current_with_span(span);
        let _guard = ctx.attach();
        // println!("execute_internal::traced");
        upstream_execute_internal(Some(execute_data), Some(return_value));
    } else {
        // println!("execute_internal::not-traced");
        upstream_execute_internal(Some(execute_data), Some(return_value));
    }
}

// This is our exec function that wraps the upstream PHP one.
// This allows us to gather our execution timing data.
unsafe extern "C" fn execute_ex(execute_data: *mut sys::zend_execute_data) {
    let execute_data = match ExecuteData::try_from_mut_ptr(execute_data) {
        Some(execute_data) => execute_data,
        None => {
            upstream_execute_ex(None);
            return;
        }
    };

    let (function_name, class_name) = match get_function_and_class_name(execute_data) {
        Ok(names) => names,
        Err(_) => (None, None), // Handle errors gracefully
    };

    if should_trace(class_name.as_deref(), function_name.as_deref()) {
        let tracer = global::tracer("php-auto-instrumentation");
        let span_name = format!(
            "{}::{}",
            class_name.as_deref().unwrap_or("<global>"),
            function_name.as_deref().unwrap_or("<anonymous>")
        );
        let span = tracer.start(span_name);
        let ctx = Context::current_with_span(span);
        let _guard = ctx.attach();
        upstream_execute_ex(Some(execute_data));
    } else {
        upstream_execute_ex(Some(execute_data));
    }

    // Run the upstream function and record the duration.
    // let start = get_unix_timestamp_micros();
    // upstream_execute_ex(Some(execute_data));
    // let end = get_unix_timestamp_micros();
}

#[inline]
fn upstream_execute_ex(execute_data: Option<&mut ExecuteData>) {
    unsafe {
        if let Some(f) = UPSTREAM_EXECUTE_EX {
            f(execute_data
                .map(ExecuteData::as_mut_ptr)
                .unwrap_or(null_mut()))
        }
    }
}

#[inline]
fn upstream_execute_internal(execute_data: Option<&mut ExecuteData>, return_value: Option<*mut sys::zval>) {
    unsafe {
        if let Some(f) = UPSTREAM_EXECUTE_INTERNAL {
            // ✅ Ensure both arguments have valid pointers before calling
            let execute_data_ptr = execute_data.map(ExecuteData::as_mut_ptr).unwrap_or(null_mut());
            let return_value_ptr = return_value.unwrap_or(null_mut());

            println!("Calling original zend_execute_internal...");
            f(execute_data_ptr, return_value_ptr);
            println!("Finished executing internal function.");
        } else {
            println!("UPSTREAM_EXECUTE_INTERNAL is None, internal function not executed.");
        }
    }
}

pub fn get_unix_timestamp_micros() -> i64 {
    let now = Utc::now();
    now.timestamp_micros()
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

fn should_trace(class_name: Option<&str>, function_name: Option<&str>) -> bool {
    match (class_name, function_name) {
        (Some("DemoClass"), Some(_)) => true,
        (None, Some("demoFunction")) => true,
        (Some("DemoClass"), None) => true,
        (None, Some("str_contains")) => true,
        _ => false,
    }
}