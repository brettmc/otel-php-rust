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

// This function swaps out the PHP exec function for our own. Allowing us to wrap it.
pub fn register_exec_functions() {
    unsafe {
        UPSTREAM_EXECUTE_EX = sys::zend_execute_ex;
        sys::zend_execute_ex = Some(execute_ex);
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

    if class_name.as_deref() == Some("DemoClass") /*&& function_name.as_deref() == Some("test")*/ {
        // println!("Matched: DemoClass::test is running!");
        let tracer = global::tracer("php-auto-instrumentation");
        let span_name = format!("{}::{}", class_name.as_deref().unwrap_or("<unknown>"), function_name.as_deref().unwrap_or("<unknown>"));
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

pub fn get_unix_timestamp_micros() -> i64 {
    let now = Utc::now();
    now.timestamp_micros()
}

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