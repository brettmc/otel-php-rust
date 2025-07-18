use phper::{
    arrays::{IterKey, ZArray},
    values::ZVal,
    sys::sapi_module,
};
use opentelemetry::{
    Array,
    KeyValue,
    StringValue,
    Value,
};
use std::ffi::CStr;

pub fn zval_to_key_value(key: &str, value: &ZVal) -> Option<KeyValue> {
    let type_info = value.get_type_info();
    if type_info.is_string() {
        return value.as_z_str().and_then(|z| z.to_str().ok()).map(|s| KeyValue::new(key.to_string(), s.to_string()));
    }
    if type_info.is_long() {
        return value.as_long().map(|v| KeyValue::new(key.to_string(), v));
    }
    if type_info.is_double() {
        return value.as_double().map(|v| KeyValue::new(key.to_string(), v));
    }
    if type_info.is_bool() {
        return value.as_bool().map(|v| KeyValue::new(key.to_string(), v));
    }
    if type_info.is_array() {
        return zval_to_vec(key, value);
    }
    None
}

pub fn zval_arr_to_key_value_vec(arr: ZArray) -> Vec<KeyValue> {
    let mut result = Vec::new();

    //iterate over zarr
    for (key, value) in arr.iter() {
        match key {
            IterKey::Index(_) => {}, // Skip integer keys
            IterKey::ZStr(zstr) => {
                if let Some(key_str) = zstr.to_str().ok().map(|s| s.to_string()) {
                    if let Some(kv) = zval_to_key_value(&key_str, value) {
                        result.push(kv);
                    }
                }
            },
        };
    }

    result
}

fn zval_to_vec(key: &str, value: &ZVal) -> Option<KeyValue> {
    let array = value.as_z_arr()?;

    let mut string_values = Vec::new();
    let mut int_values = Vec::new();
    let mut float_values = Vec::new();
    let mut bool_values = Vec::new();

    for (_, v) in array.iter() {
        if let Some(val) = v.as_z_str().and_then(|z| z.to_str().ok()) {
            string_values.push(val.to_string());
        } else if let Some(val) = v.as_long() {
            int_values.push(val);
        } else if let Some(val) = v.as_double() {
            float_values.push(val);
        } else if let Some(val) = v.as_bool() {
            bool_values.push(val);
        }
    }

    if !string_values.is_empty() {
        return Some(KeyValue::new(
            key.to_string(),
            Value::Array(Array::from(
                string_values.into_iter().map(StringValue::from).collect::<Vec<_>>(),
            )),
        ));
    } else if !int_values.is_empty() {
        return Some(KeyValue::new(
            key.to_string(),
            Value::Array(Array::from(int_values)),
        ));
    } else if !float_values.is_empty() {
        return Some(KeyValue::new(
            key.to_string(),
            Value::Array(Array::from(float_values)),
        ));
    } else if !bool_values.is_empty() {
        return Some(KeyValue::new(
            key.to_string(),
            Value::Array(Array::from(bool_values)),
        ));
    }

    None
}

pub fn get_sapi_module_name() -> String {
    unsafe { CStr::from_ptr(sapi_module.name).to_string_lossy().into_owned() }
}

pub fn get_php_version() -> String {
    let php_version = format!(
        "{}.{}.{}",
        phper::sys::PHP_MAJOR_VERSION,
        phper::sys::PHP_MINOR_VERSION,
        phper::sys::PHP_RELEASE_VERSION
    );
    php_version
}
