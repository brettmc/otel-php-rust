use phper::{
    classes::ClassEntry,
    functions::ZFunc,
    sys,
    values::{
        ExecuteData,
    },
    strings::{ZStr, ZString},
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
        if should_trace(exec_data.func()) {
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

//copied from https://github.com/apache/skywalking-php/blob/v0.8.0/src/execute.rs#L283
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

fn should_trace(func: &ZFunc) -> bool {
    // println!("should_trace");

    let function_name: ZString = func.get_function_or_method_name();
    let function_name_str = match function_name.to_str() {
        Ok(name) => name,
        Err(_) => return false, // If the function name is not valid UTF-8, return false
    };
    // println!("function_name: {:?}", function_name);
    #[cfg(feature="test")]
    let known_functions = &[
        "DemoClass::test",
        "DemoClass::inner",
        "demoFunction",
        "str_contains",
    ];
    #[cfg(not(feature="test"))]
    let known_functions: &[&str] = &[];
    if known_functions.iter().any(|&name| function_name_str == name) {
        return true;
    }

    //check for interfaces
    let ce = match func.get_class() {
        Some(class_entry) => class_entry,
        None => return false,
    };

    let known_interfaces = &["IDemo::foo", "IDemo::bar"];
    for &iface_entry in known_interfaces {
        let parts: Vec<&str> = iface_entry.split("::").collect();
        if parts.len() != 2 {
            println!("Skipping malformed interface entry: {}", iface_entry);
            continue;
        }
        let interface_name = parts[0];
        let method_name = parts[1];

        match ClassEntry::from_globals(interface_name) {
            Ok(iface_ce) => {
                // println!("interface CE found: {}", interface_name);
                if ce.is_instance_of(&iface_ce) {
                    if iface_ce.has_method(method_name) {
                        // println!("match on interface + method");
                        return true;
                    }
                }
            }
            Err(_) => {}
        }
    }

    false
}