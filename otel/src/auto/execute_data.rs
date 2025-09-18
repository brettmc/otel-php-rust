use phper::{
    alloc::{
        RefClone,
        ToRefOwned,
    },
    arrays::{ZArray, IterKey},
    eg,
    objects::ZObj,
    strings::ZStr,
    sys::{
        IS_UNDEF,
        phper_zend_add_call_flag,
        phper_zend_call_arg,
        phper_zend_call_num_args,
        phper_zend_call_may_have_undef,
        phper_zend_call_var_num,
        phper_zend_set_call_num_args,
        phper_zval_copy,
        phper_zval_undef,
        phper_zval_null,
        phper_z_type_p,
        zend_execute_data,
    },
    values::{ExecuteData, ZVal},
};
use opentelemetry::{
    KeyValue,
};
use std::{
    cell::RefCell,
    collections::HashMap,
};

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
    tracing::trace!("get_function_arguments: num_args={}", num_args);
    let mut arr = ZArray::new();
    for i in 0..num_args {
        let val = execute_data.get_parameter(i);
        arr.insert((), val.clone());
    }
    ZVal::from(arr)
}

// Returns Some(index) if the name matches a parameter, else None
fn parameter_index_by_name(execute_data: &ExecuteData, name: &str) -> Option<usize> {
    let arg_info_ptr = execute_data.common_arg_info();
    let num_args = execute_data.common_num_args() as usize;

    if !arg_info_ptr.is_null() {
        for i in 0..num_args {
            unsafe {
                let info = arg_info_ptr.add(i);
                // Convert the raw zend_string pointer to ZStr, then to &str
                let param_name_ptr = (*info).name;
                let param_name = if !param_name_ptr.is_null() {
                    ZStr::from_ptr(param_name_ptr).to_str().ok()
                } else {
                    None
                };
                if let Some(param_name) = param_name {
                    if param_name == name {
                        return Some(i);
                    }
                }
            }
        }
    }
    None
}

pub fn set_parameter_slots<'a, I>(execute_data: &mut ExecuteData, replacements: I)
where
    I: IntoIterator<Item = (IterKey<'a>, ZVal)>
{
    // Collect current arguments
    let num_args = execute_data.num_args();
    let mut args: Vec<ZVal> = (0..num_args)
        .map(|i| execute_data.get_parameter(i).clone())
        .collect();

    // Apply replacements
    for (key, value) in replacements {
        match key {
            IterKey::Index(index) => {
                let idx = index as usize;
                if idx < args.len() {
                    args[idx] = value;
                } else {
                    // Fill with UNDEF/null up to the new index
                    while args.len() < idx {
                        args.push(ZVal::from(()));
                    }
                    args.push(value);
                }
            }
            IterKey::ZStr(s) => {
                if let Some(idx) = parameter_index_by_name(execute_data, s.to_str().unwrap_or("")) {
                    if idx < args.len() {
                        args[idx] = value;
                    }
                } else {
                    tracing::warn!("pre hook unknown named arg '{}'", s.to_str().unwrap_or(""));
                }
            }
        }
    }

    // Write back all arguments
    let ptr: *mut zend_execute_data = execute_data.as_mut_ptr();
    unsafe {
        let old_num_args = phper_zend_call_num_args(ptr);
        if args.len() > old_num_args as usize {
            phper_zend_add_call_flag(ptr, phper_zend_call_may_have_undef());
            phper_zend_set_call_num_args(ptr, args.len() as u32);
            for i in old_num_args as usize..args.len() {
                phper_zval_undef(phper_zend_call_arg(ptr, i as i32));
            }
        }
        for (i, val) in args.iter().enumerate() {
            phper_zval_copy(phper_zend_call_var_num(ptr, i as i32), val.as_ptr());
        }
    }
    tracing::debug!("after: arguments: {:?}", get_function_arguments(execute_data));
}

/// Replicates otel_observer.c logic for handling missing default arguments after argument mutation.
/// Should be called after all pre hooks have run and arguments may have been added.
pub fn handle_missing_default_args(execute_data: &mut ExecuteData) {
    use phper::sys::{
        phper_zend_call_info,
        phper_zend_call_may_have_undef,
        zend_handle_undef_args,
        phper_zend_call_num_args,
        phper_zend_call_var_num,
    };
    let ptr = execute_data.as_mut_ptr();
    tracing::info!("handle_missing_default_args: called for execute_data={:p}", ptr);
    let call_info = unsafe { phper_zend_call_info(ptr) };
    let may_have_undef = unsafe { phper_zend_call_may_have_undef() };
    if (call_info & may_have_undef) != 0 {
        let eg_exception = unsafe { phper::eg!(exception) };
        unsafe {
            phper::eg!(exception) = 1_usize as *mut _;
        }
        tracing::trace!("handle_missing_default_args: EG(exception) temporarily set to invalid pointer");

        let undef_result = unsafe { zend_handle_undef_args(ptr) };
        tracing::trace!("handle_missing_default_args: zend_handle_undef_args returned {}", undef_result);

        if undef_result == unsafe { phper::sys::phper_zend_result_failure() } {
            let arg_count = unsafe { phper_zend_call_num_args(ptr) };
            for i in 0..arg_count {
                let arg = unsafe { phper_zend_call_var_num(ptr, i as i32) };
                let type_info = unsafe { phper_z_type_p(arg) };
                tracing::trace!(
                    "handle_missing_default_args: arg[{}] at {:p} type_info={}",
                    i, arg, type_info
                );
                if type_info != IS_UNDEF.try_into().unwrap() {
                    tracing::trace!("handle_missing_default_args: arg[{}] is defined, skipping", i);
                    continue;
                }
                tracing::trace!("handle_missing_default_args: arg[{}] IS_UNDEF, setting to NULL", i);
                unsafe { phper_zval_null(arg); }
            }
        }
        unsafe {
            phper::eg!(exception) = eg_exception;
        }
        tracing::trace!("handle_missing_default_args: EG(exception) restored to {:p}", eg_exception);
    }
}
