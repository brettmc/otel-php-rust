use phper::{
    alloc::{
        RefClone,
        ToRefOwned,
    },
    arrays::ZArray,
    eg,
    objects::ZObj,
    strings::ZStr,
    sys::{
        phper_zend_add_call_flag,
        phper_zend_call_arg,
        phper_zend_call_may_have_undef,
        phper_zend_call_var_num,
        phper_zend_call_num_args,
        phper_zval_null,
        zend_execute_data,
        phper_zval_copy,
        phper_zend_set_call_num_args,
        phper_zval_undef,
    },
    values::{ExecuteData, ZVal},
};
use opentelemetry::{
    KeyValue,
};
use std::collections::HashMap;
use std::cell::RefCell;

// Storage for communication between pre and post hooks, using exec_data as key
thread_local! {
    static EXEC_DATA_FLAGS: RefCell<HashMap<usize, bool>> = RefCell::new(HashMap::new());
}

/// Set a flag. If you set a flag (eg in a pre hook), ensure you remove it in the post hook.
pub fn set_exec_data_flag(exec_data: *mut ExecuteData, value: bool) {
    let key = exec_data as usize;
    EXEC_DATA_FLAGS.with(|map| {
        map.borrow_mut().insert(key, value);
    });
}

// Get a flag
pub fn get_exec_data_flag(exec_data: *mut ExecuteData) -> Option<bool> {
    let key = exec_data as usize;
    EXEC_DATA_FLAGS.with(|map| {
        map.borrow().get(&key).copied()
    })
}

// Remove a flag (cleanup)
pub fn remove_exec_data_flag(exec_data: *mut ExecuteData) {
    let key = exec_data as usize;
    EXEC_DATA_FLAGS.with(|map| {
        map.borrow_mut().remove(&key);
    });
}

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

pub fn get_fqn(execute_data: &ExecuteData) -> Option<String> {
    let (function_name, class_name) = get_function_and_class_name(execute_data).unwrap_or((None, None));

    match (class_name, function_name) {
        (Some(cls), Some(func)) => Some(format!("{}::{}", cls, func)),
        (None, Some(func)) => Some(func),
        _ => None,
    }
}

pub fn get_file_and_line(execute_data: &ExecuteData) -> Option<(String, u32)> {
    let filename = execute_data.func().get_filename();
    let lineno = execute_data.func().get_line_start();
    if filename.is_some() && lineno.is_some() {
        let file = filename.unwrap().to_str().unwrap_or("").to_string();
        let line = lineno.unwrap();
        Some((file, line))
    } else {
        None
    }
}

pub fn get_this_or_called_scope(execute_data: &mut ExecuteData) -> ZVal {
    let function = execute_data.func();

    if function.is_static() {
        // Get called scope (if available)
        if let Some(called_scope) = execute_data.get_called_scope() {
            // Return class name as string
            return ZVal::from(called_scope.get_name().to_str().unwrap_or(""));
        }
    } else {
        // Get $this object (if available)
        if let Some(zobj) = execute_data.get_this_mut() {
            let mut owned = zobj.to_ref_owned();
            return ZVal::from(owned.ref_clone());
        }
    }
    ZVal::from(())
}

pub fn get_global_exception() -> Option<&'static mut ZObj> {
    unsafe {
        let obj = ZObj::try_from_mut_ptr(eg!(exception))?;
        let name = obj.get_class().get_name().to_str().unwrap_or_default();
        if name == "UnwindExit" {
            return None;
        }
        Some(obj)
    }
}

// Default auto-instrumentation attributes from ExecuteData
pub fn get_default_attributes(execute_data: &ExecuteData) -> Vec<KeyValue> {
    let mut attributes = vec![];
    if let Some(fqn) = get_fqn(execute_data) {
        attributes.push(KeyValue::new("code.function.name".to_string(), fqn));
    }
    if let Some((file, line)) = get_file_and_line(execute_data) {
        attributes.push(KeyValue::new("code.file.path".to_string(), file));
        attributes.push(KeyValue::new("code.line.number".to_string(), line as i64));
    }

    attributes
}

/// Retrieve all arguments to a function call as a ZVal representing an array
pub fn get_function_arguments(execute_data: &ExecuteData) -> ZVal {
    let num_args = execute_data.num_args();
    let mut arr = ZArray::new();
    for i in 0..num_args {
        let val = execute_data.get_parameter(i);
        arr.insert((), val.clone());
    }
    ZVal::from(arr)
}

pub fn ensure_parameter_slot(execute_data: &mut ExecuteData, index: usize) {
    let num_args = execute_data.num_args();
    if index >= num_args {
        tracing::debug!("Extending parameter slots from {} to {}", num_args, index + 1);
        let ptr: *mut zend_execute_data = execute_data.as_mut_ptr();
        unsafe {
            let params_ptr = phper_zend_call_var_num(ptr, index as i32);
            phper_zval_null(params_ptr);
            (*ptr).This.u2.num_args = (index + 1) as u32;
        }
    }
}

pub fn set_parameter_slot(execute_data: &mut ExecuteData, index: usize, value: ZVal) {
    let ptr: *mut zend_execute_data = execute_data.as_mut_ptr();
    unsafe {
        let num_args = phper_zend_call_num_args(ptr).try_into().unwrap();
        let _arg_ptr = phper_zend_call_var_num(ptr, index as i32);
        if index >= num_args {
            tracing::debug!("Extending parameter slots from {} to {}", num_args, index + 1);
            phper_zval_undef(phper_zend_call_arg(ptr, (index+1) as i32));
            phper_zend_add_call_flag(ptr, phper_zend_call_may_have_undef());
            phper_zend_set_call_num_args(ptr, (index + 1) as u32);
            phper_zval_copy(phper_zend_call_var_num(ptr, (index+1) as i32), value.as_ptr());
        } else {
            tracing::debug!("Setting parameter slot {} (num_args={})", index, num_args);
            phper_zval_copy(phper_zend_call_var_num(ptr, index as i32), value.as_ptr());
        }
    }
}

// let num_args = self.num_args();
// if index >= num_args {
// unsafe {
// let params_ptr = phper_zend_call_var_num(self.as_mut_ptr(), index as i32);
// phper_zval_null(params_ptr);
// (*self.inner.func).common.num_args = (index + 1) as u32;
// }
// }

