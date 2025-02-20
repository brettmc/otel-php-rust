// copied from https://github.com/skpr/compass/blob/v1.2.0/extension/src/execute.rs
use chrono::prelude::*;
use phper::{
    sys,
    values::{
        ExecuteData,
    },
    strings::ZStr,
};
use std::ptr::null_mut;
use opentelemetry::{
    global,
    Context,
    trace::{
        Span,
        Tracer,
        TraceContextExt,
    }
};

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
use std::ffi::CStr;
use opentelemetry::global::BoxedSpan;
use opentelemetry::ContextGuard;

pub unsafe extern "C" fn observer_begin(execute_data: *mut sys::zend_execute_data) {
    if let Some(execute_data) = ExecuteData::try_from_mut_ptr(execute_data) {
        let (function_name, class_name) = match get_function_and_class_name(execute_data) {
            Ok(names) => names,
            Err(_) => (None, None), // Handle errors gracefully
        };
        let span_name = format!(
            "{}::{}",
            class_name.as_deref().unwrap_or("<global>"),
            function_name.as_deref().unwrap_or("<anonymous>")
        );
        println!("[BEGIN: {}]", span_name.clone());
    }
}

pub unsafe extern "C" fn observer_end(
    execute_data: *mut sys::zend_execute_data,
    return_value: *mut sys::zval,
) {
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

        let ret_type = if !return_value.is_null() {
            let type_cstr = CStr::from_ptr(sys::zend_zval_type_name(return_value));
            type_cstr.to_string_lossy().into_owned()
        } else {
            "null".to_owned()
        };
        println!("[END {}(): {}]", span_name, ret_type);
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