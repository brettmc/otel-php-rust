use phper::{
    eg,
    objects::ZObj,
    sys,
    strings::{ZStr},
    values::{
        ExecuteData,
    }
};
use opentelemetry::{
    KeyValue,
};

//copied from https://github.com/apache/skywalking-php/blob/v0.8.0/src/execute.rs#L283
pub fn get_function_and_class_name(
    execute_data: &ExecuteData,
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

pub fn get_fqn(execute_data: &ExecuteData) -> String {
    let (function_name, class_name) = get_function_and_class_name(execute_data).unwrap_or((None, None));

    match (class_name, function_name) {
        (Some(cls), Some(func)) => format!("{}::{}", cls, func),
        (None, Some(func)) => func,
        _ => "<unknown>".to_string(),
    }
}

//TODO get these through ExecuteData?
pub unsafe fn get_file_and_line(execute_data: &ExecuteData) -> Option<(String, u32)> {
    let zend_execute_data = execute_data.as_ptr();

    if zend_execute_data.is_null() {
        return None;
    }

    let func = (*zend_execute_data).func;
    if func.is_null() {
        return None;
    }

    let func = &*func;

    // Ensure it's a user-defined function before accessing op_array
    if func.type_ as u32 != sys::ZEND_USER_FUNCTION {
        return None; // Not a user-defined function, no file/line info available
    }

    let op_array = &func.op_array;

    let file_name = if !op_array.filename.is_null() {
        let zend_filename = &*op_array.filename;
        let c_str = std::ffi::CStr::from_ptr(zend_filename.val.as_ptr());
        c_str.to_string_lossy().into_owned()
    } else {
        "<unknown>".to_string()
    };

    let line_number = if !(*zend_execute_data).opline.is_null() {
        (*(*zend_execute_data).opline).lineno
    } else {
        0
    };

    Some((file_name, line_number))
}

pub fn get_global_exception() -> Option<&'static mut ZObj> {
    unsafe { ZObj::try_from_mut_ptr(eg!(exception)) }
}

// Default auto-instrumentation attributes from ExecuteData
pub fn get_default_attributes(execute_data: &ExecuteData) -> Vec<KeyValue> {
    let mut attributes = vec![KeyValue::new("code.function.name".to_string(), get_fqn(execute_data))];
    unsafe {
        if let Some((file, line)) = get_file_and_line(execute_data) {
            attributes.push(KeyValue::new("code.file.path".to_string(), file));
            attributes.push(KeyValue::new("code.line.number".to_string(), line as i64));
        }
    }

    attributes
}