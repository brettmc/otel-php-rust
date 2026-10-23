use opentelemetry::{Array, KeyValue, StringValue, Value};
use phper::{
    arrays::{IterKey, ZArray},
    sys::sapi_module,
    values::ZVal,
};
use std::ffi::CStr;

/// Convert a ZVal to a single KeyValue pair based on its type.
pub fn zval_to_key_value(key: &str, value: &ZVal) -> Option<KeyValue> {
    let type_info = value.get_type_info();
    if type_info.is_string() {
        return value
            .as_z_str()
            .and_then(|z| z.to_str().ok())
            .map(|s| KeyValue::new(key.to_string(), s.to_string()));
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

/// Convert a ZArray to a vector of KeyValue pairs.
pub fn zval_arr_to_key_value_vec(arr: ZArray) -> Vec<KeyValue> {
    let mut result = Vec::new();

    //iterate over zarr
    for (key, value) in arr.iter() {
        match key {
            IterKey::Index(_) => {} // Skip integer keys
            IterKey::ZStr(zstr) => {
                if let Ok(key_str) = zstr.to_str()
                    && let Some(kv) = zval_to_key_value(key_str, value)
                {
                    result.push(kv);
                }
            }
        };
    }

    result
}

/// Insert a `KeyValue` into a PHP array, converting the value to its PHP equivalent.
///
/// Scalars become the corresponding PHP scalar and arrays become a packed PHP array. A
/// value of a kind not representable here is skipped rather than inserted as null, so a
/// caller reading the array sees the key as absent.
pub fn insert_attribute(arr: &mut ZArray, kv: &KeyValue) {
    let key = kv.key.as_str();
    match &kv.value {
        Value::String(s) => arr.insert(key, s.as_str()),
        Value::I64(i) => arr.insert(key, *i),
        Value::F64(f) => arr.insert(key, *f),
        Value::Bool(b) => arr.insert(key, *b),
        Value::Array(values) => {
            let mut arr_values = ZArray::new();
            match values {
                Array::Bool(vec) => {
                    for v in vec {
                        arr_values.insert((), *v);
                    }
                }
                Array::I64(vec) => {
                    for v in vec {
                        arr_values.insert((), *v);
                    }
                }
                Array::F64(vec) => {
                    for v in vec {
                        arr_values.insert((), *v);
                    }
                }
                Array::String(vec) => {
                    for v in vec {
                        arr_values.insert((), v.as_str());
                    }
                }
                // `Array` is non-exhaustive upstream; skip kinds we cannot represent.
                _ => return,
            }
            arr.insert(key, arr_values);
        }
        // `Value` is non-exhaustive upstream; skip kinds we cannot represent.
        _ => {}
    }
}

/// Convert a set of `KeyValue`s into a PHP array, skipping any unrepresentable values.
pub fn attributes_to_zarray<'a>(attributes: impl IntoIterator<Item = &'a KeyValue>) -> ZArray {
    let mut arr = ZArray::new();
    for kv in attributes {
        insert_attribute(&mut arr, kv);
    }
    arr
}

/// Get the name of the SAPI module.
pub fn get_sapi_module_name() -> String {
    unsafe {
        CStr::from_ptr(sapi_module.name)
            .to_string_lossy()
            .into_owned()
    }
}

/// Get the PHP version as a string.
pub fn get_php_version() -> String {
    let php_version = format!(
        "{}.{}.{}",
        phper::sys::PHP_MAJOR_VERSION,
        phper::sys::PHP_MINOR_VERSION,
        phper::sys::PHP_RELEASE_VERSION
    );
    php_version
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
                string_values
                    .into_iter()
                    .map(StringValue::from)
                    .collect::<Vec<_>>(),
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
