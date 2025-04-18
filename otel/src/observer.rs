use phper::{
    eg,
    objects::ZObj,
    sys,
    strings::{ZStr},
    values::{
        ExecuteData,
        ZVal,
    }
};
use lazy_static::lazy_static;
use crate::{
    logging,
    trace::plugin::{
        FunctionObserver,
        SpanDetails,
    },
    tracer_provider,
    PluginManager
};
use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock, Mutex},
    cell::RefCell,
};
use opentelemetry::{
    Context,
    ContextGuard,
    KeyValue,
    trace::{
        Tracer,
        TraceContextExt,
        TracerProvider,
    },
};

static FUNCTION_OBSERVERS: OnceLock<RwLock<HashMap<String, FunctionObserver>>> = OnceLock::new();

lazy_static! {
    static ref PLUGIN_MANAGER: Mutex<Option<PluginManager>> = Mutex::new(None);
}

thread_local! {
    static CONTEXT_GUARD_MAP: RefCell<HashMap<usize, ContextGuard>> = RefCell::new(HashMap::new());
}

fn store_guard(exec_data: *mut ExecuteData, guard: ContextGuard) {
    let key = exec_data as *const ExecuteData as usize;
    CONTEXT_GUARD_MAP.with(|map| {
        map.borrow_mut().insert(key, guard);
    });
}

fn take_guard(exec_data: *mut ExecuteData) -> Option<ContextGuard> {
    let key = exec_data as *const ExecuteData as usize;
    CONTEXT_GUARD_MAP.with(|map| map.borrow_mut().remove(&key))
}

pub fn init(plugin_manager: PluginManager) {
    logging::print_message("PluginManager::init".to_string());
    let mut manager_lock = PLUGIN_MANAGER.lock().unwrap();
    *manager_lock = Some(plugin_manager);
    FUNCTION_OBSERVERS.get_or_init(|| RwLock::new(HashMap::new()));
}

pub unsafe extern "C" fn observer_instrument(execute_data: *mut sys::zend_execute_data) -> sys::zend_observer_fcall_handlers {
    if let Some(exec_data) = ExecuteData::try_from_mut_ptr(execute_data) {
        let fqn = get_fqn(exec_data);
        //tracing::trace!("observer::observer_instrument checking: {}", fqn);
        let manager_lock = PLUGIN_MANAGER.lock().unwrap();
        if let Some(plugin_manager) = manager_lock.as_ref() {
            if let Some(observer) = plugin_manager.get_function_observer(exec_data) {
                let observers = FUNCTION_OBSERVERS.get().expect("Function observer not initialized");
                let fqn = fqn.to_string();
                let mut lock = observers.write().unwrap();
                lock.insert(fqn, observer);

                static mut HANDLERS: sys::zend_observer_fcall_handlers = sys::zend_observer_fcall_handlers {
                    begin: Some(pre_observe_c_function),
                    end: Some(post_observe_c_function),
                };

                return unsafe { HANDLERS };
            }
        } else {
            tracing::error!("Plugin manager not available");
        }
    }

    sys::zend_observer_fcall_handlers {
        begin: None,
        end: None,
    }
}

#[no_mangle]
pub unsafe extern "C" fn pre_observe_c_function(execute_data: *mut sys::zend_execute_data) {
    if let Some(exec_data) = ExecuteData::try_from_mut_ptr(execute_data) {
        let fqn = get_fqn(exec_data);

        let observers = FUNCTION_OBSERVERS.get().expect("Function observer not initialized");
        let lock = observers.read().unwrap();
        if let Some(observer) = lock.get(&fqn) {
            if observer.has_hooks() {
                let tracer = tracer_provider::get_tracer_provider().tracer("php-auto-instrumentation");
                let mut span_details = SpanDetails::new(fqn.clone(), get_default_attributes(exec_data));
                for hook in observer.pre_hooks() {
                    tracing::trace!("running pre hook: {}", fqn);
                    hook(&mut *exec_data, &mut span_details);
                }
                let span_builder = tracer.span_builder(span_details.name());
                let span_builder = span_builder.with_attributes(span_details.attributes());
                let span_builder = span_builder.with_kind(span_details.kind());
                let span = tracer.build_with_context(span_builder, &Context::current());
                let ctx = Context::current_with_span(span);
                let guard = ctx.attach();
                store_guard(exec_data, guard);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn post_observe_c_function(execute_data: *mut sys::zend_execute_data, retval: *mut sys::zval) {
    if let Some(exec_data) = ExecuteData::try_from_mut_ptr(execute_data) {
        let fqn = get_fqn(exec_data);

        let observers = FUNCTION_OBSERVERS.get().expect("Function observer not initialized");
        let lock = observers.read().unwrap();
        if let Some(observer) = lock.get(&fqn) {
            if let Some(guard) = take_guard(exec_data) {
                let context = Context::current();
                let mut span_ref = context.span();

                //TODO use Option<ZVal> ??
                let retval = if retval.is_null() {
                    &mut ZVal::from(())
                } else {
                    (retval as *mut ZVal).as_mut().unwrap()
                };

                for hook in observer.post_hooks() {
                    tracing::trace!("running post hook: {}", fqn);
                    hook(&mut *exec_data, &mut span_ref, retval, get_global_exception());
                }
                // Dropping the guard detaches the context and finishes the span.
                drop(guard);
            } else {
                tracing::debug!("No active opentelemetry span guard found for execute_data at: {:p}", exec_data);
            }
        }
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

fn get_fqn(execute_data: &mut ExecuteData) -> String {
    let (function_name, class_name) = get_function_and_class_name(execute_data).unwrap_or((None, None));

    match (class_name, function_name) {
        (Some(cls), Some(func)) => format!("{}::{}", cls, func),
        (None, Some(func)) => func,
        _ => "<unknown>".to_string(),
    }
}

fn get_default_attributes(execute_data: &mut ExecuteData) -> Vec<KeyValue> {
    let mut attributes = vec![KeyValue::new("code.function.name".to_string(), get_fqn(execute_data))];
    unsafe {
        if let Some((file, line)) = get_file_and_line(execute_data) {
            //println!("Executing file: {} at line: {}", file, line);
            attributes.push(KeyValue::new("code.file.path".to_string(), file));
            attributes.push(KeyValue::new("code.line.number".to_string(), line as i64));
        }
    }

    attributes
}

//TODO get these through ExecuteData?
unsafe fn get_file_and_line(execute_data: &ExecuteData) -> Option<(String, u32)> {
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

fn get_global_exception() -> Option<&'static mut ZObj> {
    unsafe { ZObj::try_from_mut_ptr(eg!(exception)) }
}
