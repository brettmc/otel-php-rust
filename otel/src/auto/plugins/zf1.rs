use crate::{
    auto::{
        execute_data,
        plugin::{Handler, HandlerList, HandlerSlice, HandlerCallbacks, Plugin},
        utils,
    },
    config::trace_attributes,
    context::storage::{take_guard},
    trace::local_root_span::get_local_root_span_context,
    tracer_provider,
};
use opentelemetry::{
    KeyValue,
    trace::{
        SpanContext,
        TraceContextExt,
        TracerProvider,
    },
};
use opentelemetry_semantic_conventions as SemConv;
use std::{
    sync::Arc,
    collections::HashMap,
    sync::Mutex,
};
use lazy_static::lazy_static;
use phper::{
    alloc::ToRefOwned,
    objects::ZObj,
    values::{
        ExecuteData,
        ZVal,
    },
};

// Zend Framework 1 (ZF1) plugin for OpenTelemetry PHP auto-instrumentation.
// db connections are not tracked, as the _connect method is called before every db operation,
// which uses internal functions (can not be instrumented with php <8.2)

struct ConnectionInfo {
    attributes: Vec<KeyValue>,
    span_context: SpanContext,
}
struct StatementInfo {
    attributes: Vec<KeyValue>,
    span_name: String,
    span_context: SpanContext,
}

lazy_static! {
    static ref CONNECTION_ATTRS: Mutex<HashMap<usize, ConnectionInfo>> = Mutex::new(HashMap::new());
    static ref STATEMENT_ATTRS: Mutex<HashMap<usize, StatementInfo>> = Mutex::new(HashMap::new());
}

// Helper to get object id (pointer address)
fn get_object_id(obj: &ZObj) -> usize {
    obj as *const _ as usize
}

pub struct Zf1Plugin {
    handlers: HandlerList,
}

impl Zf1Plugin {
    pub fn new() -> Self {
        Self {
            handlers: vec![
                Arc::new(Zf1RouteHandler),
                Arc::new(Zf1SendResponseHandler),
                Arc::new(Zf1AdapterConnectHandler),
                Arc::new(Zf1AdapterPrepareHandler),
                Arc::new(Zf1StatementExecuteHandler),
            ],
        }
    }
}

impl Plugin for Zf1Plugin {
    fn get_handlers(&self) -> &HandlerSlice {
        &self.handlers
    }
    fn get_name(&self) -> &str {
        "zf1"
    }
    fn request_shutdown(&self) {
        STATEMENT_ATTRS.lock().unwrap().clear();
        CONNECTION_ATTRS.lock().unwrap().clear();
    }
}

pub struct Zf1RouteHandler;

impl Handler for Zf1RouteHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some("Zend_Controller_Router_Interface".to_string()), "route".to_string()),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: None,
            post_observe: Some(Box::new(|exec_data, retval, exception| unsafe {
                Self::post_callback(exec_data, retval, exception)
            })),
        }
    }
}

impl Zf1RouteHandler {
    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        retval: Option<&mut ZVal>,
        _exception: Option<&mut ZObj>
    ) {
        tracing::debug!("Auto::Zf1::post (Router_Interface::route)");
        let ctx = match get_local_root_span_context() {
            Some(ctx) => ctx,
            None => {
                tracing::debug!("Auto::Zf1::post (Router_Interface::route) - no local root span found, skipping");
                return;
            }
        };
        tracing::debug!("Auto::Zf1::post - got local root span context");
        ctx.span().set_attribute(KeyValue::new(trace_attributes::PHP_FRAMEWORK_NAME, "zf1"));

        let exec_data_ref = unsafe { &mut *exec_data };
        let zf1_request_zval = match retval {
            Some(rv) if rv.get_type_info() != phper::types::TypeInfo::NULL => rv,
            _ => {
                tracing::debug!("Auto::Zf1::post (Router_Interface::route) - no return value found, getting first parameter");
                exec_data_ref.get_mut_parameter(0)
            }
        };

        if let Some(zf1_request_obj) = zf1_request_zval.as_mut_z_obj() {
            tracing::debug!("Auto::Zf1::converted zf1_request_obj to ZObj");
            let method = zf1_request_obj
                .call("getMethod", [])
                .ok()
                .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())));

            let module = zf1_request_obj
                .call("getModuleName", [])
                .ok()
                .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())));

            let controller = zf1_request_obj
                .call("getControllerName", [])
                .ok()
                .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())));

            let action = zf1_request_obj
                .call("getActionName", [])
                .ok()
                .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())));

            let span_name = format!(
                "{} {}/{}/{}",
                method.as_deref().unwrap_or("GET"),
                module.as_deref().unwrap_or("default"),
                controller.as_deref().unwrap_or("unknown_controller"),
                action.as_deref().unwrap_or("unknown_action")
            );

            tracing::debug!("Auto::Zf1::updateName (Router_Interface::route)");
            ctx.span().update_name(span_name);
            if let Some(module) = &module {
                ctx.span().set_attribute(KeyValue::new(trace_attributes::PHP_FRAMEWORK_MODULE_NAME, module.clone()));
            }
            if let Some(controller) = &controller {
                ctx.span().set_attribute(KeyValue::new(trace_attributes::PHP_FRAMEWORK_CONTROLLER_NAME, controller.clone()));
            }
            if let Some(action) = &action {
                ctx.span().set_attribute(KeyValue::new(trace_attributes::PHP_FRAMEWORK_ACTION_NAME, action.clone()));
            }
        } else {
            tracing::debug!("Auto::Zf1::post - zf1_request_zval could not be converted to ZObj");
        }
    }
}

pub struct Zf1SendResponseHandler;
impl Handler for Zf1SendResponseHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some("Zend_Controller_Response_Abstract".to_string()), "sendResponse".to_string()),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: None,
            post_observe: Some(Box::new(|exec_data, retval, exception| unsafe {
                Self::post_callback(exec_data, retval, exception)
            })),
        }
    }
}

impl Zf1SendResponseHandler {
    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        _retval: Option<&mut ZVal>,
        _exception: Option<&mut ZObj>
    ) {
        tracing::debug!("Auto::Zf1::post (Zend_Controller_Response_Abstract::sendResponse)");

        let exec_data_ref = unsafe { &mut *exec_data };
        if let Some(this_obj) = exec_data_ref.get_this_mut() {
            let http_response_code = this_obj.call("getHttpResponseCode", [])
                .ok()
                .and_then(|zv| zv.as_long())
                .unwrap_or(200);
            let is_exception = this_obj.call("isException", [])
                .ok()
                .and_then(|zv| zv.as_bool())
                .unwrap_or(false);
            if is_exception {
                let ctx = match get_local_root_span_context() {
                    Some(ctx) => ctx,
                    None => {
                        return;
                    }
                };
                let mut exceptions = this_obj.call("getException", [])
                    .ok()
                    .and_then(|zv| zv.as_z_arr().map(|arr| arr.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()))
                    .unwrap_or_default();

                let mut status_description = "exception".to_string();

                if let Some(exception) = exceptions.first_mut() {
                    if let Some(exception_obj) = exception.as_mut_z_obj() {
                        if let Ok(throwable) = phper::errors::ThrowObject::new(exception_obj.to_ref_owned()) {
                            ctx.span().record_error(&throwable);
                        }
                        status_description = exception_obj.call("getMessage", [])
                            .ok()
                            .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())))
                            .unwrap_or(status_description);
                    }
                }
                if http_response_code >= 500 {
                    ctx.span().set_status(opentelemetry::trace::Status::error(status_description));
                }
            }
        }
    }
}

pub struct Zf1AdapterConnectHandler;

impl Handler for Zf1AdapterConnectHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some("Zend_Db_Adapter_Abstract".to_string()), "_connect".to_string()),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Box::new(|exec_data| unsafe {
                Self::pre_callback(exec_data)
            })),
            post_observe: Some(Box::new(|exec_data, retval, exception| unsafe {
                Self::post_callback(exec_data, retval, exception)
            })),
        }
    }
}

impl Zf1AdapterConnectHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        let tracer = tracer_provider::get_tracer_provider().tracer("php.otel.auto.zf1.db");
        let exec_data_ref = unsafe {&mut *exec_data};
        let this_obj = exec_data_ref.get_this_mut().unwrap();

        let connection = this_obj.get_property("_connection");
        tracing::debug!("Zf1AdapterConnectHandler: connection type: {:?}", connection.get_type_info());
        let (_function_name, class_name) = execute_data::get_function_and_class_name(unsafe {&mut *exec_data})
            .unwrap_or((None, None));
        let is_abstract = class_name.as_deref().unwrap_or("").ends_with("_Abstract");
        tracing::debug!("Zf1AdapterConnectHandler: class_name: {:?}, is_abstract: {:?}", class_name, is_abstract);

        let should_start_span = !is_abstract && connection.get_type_info() == phper::types::TypeInfo::NULL;
        if should_start_span {
            let span_name = "connect".to_string();
            let mut execute_attributes = vec![];
            let mut attributes = vec![];
            if class_name.is_some() {
                execute_attributes.push(KeyValue::new(SemConv::trace::DB_SYSTEM_NAME, map_adapter_class_to_db_system(&class_name.unwrap())));
            }

            let config = this_obj.get_property("_config");
            if let Some(arr) = config.as_z_arr() {
                if let Some(dbname) = arr.get("dbname") {
                    tracing::debug!("Database: {:?}", dbname);
                    execute_attributes.push(KeyValue::new(
                        SemConv::trace::DB_NAMESPACE,
                        dbname.as_z_str()
                            .and_then(|s| s.to_str().ok())
                            .unwrap_or_default()
                            .to_string()
                    ));
                }
            }
            attributes.extend_from_slice(&execute_attributes);

            utils::start_and_activate_span(tracer, &span_name, attributes, exec_data, opentelemetry::trace::SpanKind::Client);

            let connection_id = get_object_id(this_obj);
            CONNECTION_ATTRS.lock().unwrap().insert(connection_id, ConnectionInfo {
                attributes: execute_attributes.clone(),
                span_context: opentelemetry::Context::current().span().span_context().clone(),
            });
        }
        tracing::debug!("Zf1AdapterConnectHandler: should_start_span: {}", should_start_span);
        execute_data::set_exec_data_flag(exec_data, should_start_span);
    }
    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        _retval: Option<&mut ZVal>,
        exception: Option<&mut ZObj>
    ) {
        let mut _guard = None;
        let did_start_span = execute_data::get_exec_data_flag(exec_data).unwrap_or(false);
        if did_start_span {
            _guard = take_guard(exec_data);
        }
        if let Some(exception) = exception {
            utils::record_exception(&opentelemetry::Context::current(), exception);
        }

        execute_data::remove_exec_data_flag(exec_data);
    }
}

pub struct Zf1AdapterPrepareHandler;

impl Handler for Zf1AdapterPrepareHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some("Zend_Db_Adapter_Abstract".to_string()), "prepare".to_string()),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Box::new(|exec_data| unsafe {
                Self::pre_callback(exec_data)
            })),
            post_observe: Some(Box::new(|exec_data, retval, exception| unsafe {
                Self::post_callback(exec_data, retval, exception)
            })),
        }
    }
}

impl Zf1AdapterPrepareHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        let tracer = tracer_provider::get_tracer_provider().tracer("php.otel.auto.zf1.db");
        let exec_data_ref = unsafe {&mut *exec_data};
        let mut span_name = "prepare".to_string();
        let mut attributes = vec![];
        let sql_zval: &mut ZVal = exec_data_ref.get_mut_parameter(0);
        if let Some(sql_str) = sql_zval.as_z_str() {
            if let Ok(sql) = sql_str.to_str() {
                // Add SQL query as an attribute
                attributes.push(KeyValue::new(SemConv::trace::DB_QUERY_TEXT, sql.to_string()));
                let sql_name = utils::extract_span_name_from_sql(&sql)
                    .unwrap_or_else(|| "OTHER".to_string());
                span_name = format!("prepare {}", sql_name);
            }
        } else {
            tracing::warn!("Zf1AdapterPrepareHandler: SQL parameter is not a string");
        }

        utils::start_and_activate_span(tracer, &span_name, attributes, exec_data, opentelemetry::trace::SpanKind::Client);
    }
    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        retval: Option<&mut ZVal>,
        exception: Option<&mut ZObj>
    ) {
        let _guard = take_guard(exec_data);
        if let Some(exception) = exception {
            utils::record_exception(&opentelemetry::Context::current(), exception);
        }
        //prepared statement is the return value
        if let Some(retval) = retval {
            if let Some(statement_obj) = retval.as_mut_z_obj() {
                let mut execute_attributes = vec![];
                let exec_data_ref = unsafe {&mut *exec_data};
                if let Some(this_obj) = exec_data_ref.get_this_mut() {
                    let id = get_object_id(this_obj);
                    tracing::debug!("Zf1AdapterPrepareHandler: object id: {}", id);
                    if let Some(info) = CONNECTION_ATTRS.lock().unwrap().get(&id) {
                        execute_attributes.extend_from_slice(&info.attributes);
                        let link = info.span_context.clone();
                        let ctx = opentelemetry::Context::current();
                        let span = ctx.span();
                        span.add_link(link, vec![]);
                    }
                }


                let exec_data_ref = unsafe { &mut *exec_data };
                let sql_zval: &mut ZVal = exec_data_ref.get_mut_parameter(0);
                if let Some(sql_str) = sql_zval.as_z_str() {
                    if let Ok(sql) = sql_str.to_str() {
                        let sql_name = utils::extract_span_name_from_sql(&sql)
                            .unwrap_or_else(|| "OTHER".to_string());
                        let execute_span_name = sql_name.clone();
                        let prepare_span_name = format!("prepare {}", sql_name.clone());
                        let ctx = opentelemetry::Context::current();
                        let span = ctx.span();
                        span.update_name(prepare_span_name);
                        execute_attributes.push(KeyValue::new(SemConv::trace::DB_QUERY_TEXT, sql.to_string()));
                        span.set_attributes(execute_attributes.clone());
                        let id = get_object_id(statement_obj);
                        // Add SQL query as an attribute
                        STATEMENT_ATTRS.lock()
                            .unwrap()
                            .insert(
                                id,
                                StatementInfo{
                                    attributes: execute_attributes.clone(),
                                    span_name: execute_span_name,
                                    span_context: opentelemetry::Context::current().span().span_context().clone(),
                                });
                    }
                } else {
                    tracing::warn!("Zf1AdapterPrepareHandler: SQL parameter is not a string");
                }
            }
        }
    }
}

pub struct Zf1StatementExecuteHandler;

impl Handler for Zf1StatementExecuteHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some("Zend_Db_Statement_Interface".to_string()), "execute".to_string()),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Box::new(|exec_data| unsafe {
                Self::pre_callback(exec_data)
            })),
            post_observe: Some(Box::new(|exec_data, retval, exception| unsafe {
                Self::post_callback(exec_data, retval, exception)
            })),
        }
    }
}

impl Zf1StatementExecuteHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        let tracer = tracer_provider::get_tracer_provider().tracer("php.otel.auto.zf1.db");
        let exec_data_ref = unsafe { &mut *exec_data };
        let mut attributes = vec![];
        let mut span_name = "Statement::execute".to_string();
        let mut link = None;

        if let Some(this_obj) = exec_data_ref.get_this_mut() {
            let id = get_object_id(this_obj);
            tracing::debug!("Zf1StatementExecuteHandler: object id: {}", id);
            if let Some(info) = STATEMENT_ATTRS.lock().unwrap().get(&id) {
                attributes.extend_from_slice(&info.attributes);
                span_name = info.span_name.clone();
                link = Some(info.span_context.clone());
            }
        }

        utils::start_and_activate_span(tracer, &span_name, attributes, exec_data, opentelemetry::trace::SpanKind::Client);
        if link.is_some() {
            opentelemetry::Context::current()
                .span()
                .add_link(link.unwrap(), vec![]);
        }
    }
    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        _retval: Option<&mut ZVal>,
        exception: Option<&mut ZObj>
    ) {
        let _guard = take_guard(exec_data);
        if let Some(exception) = exception {
            utils::record_exception(&opentelemetry::Context::current(), exception);
        }
    }
}

fn map_adapter_class_to_db_system(class_name: &str) -> &'static str {
    match class_name {
        "Zend_Db_Adapter_Pdo_Mysql" => "mysql",
        "Zend_Db_Adapter_Pdo_Pgsql" => "postgresql",
        "Zend_Db_Adapter_Pdo_Sqlite" => "sqlite",
        "Zend_Db_Adapter_Pdo_Oci" => "oracle.db",
        "Zend_Db_Adapter_Pdo_Ibm" => "ibm.db2",
        "Zend_Db_Adapter_Pdo_Mssql" => "microsoft.sql_server",
        "Zend_Db_Adapter_Mysqli" => "mysql",
        "Zend_Db_Adapter_Oracle" => "oracle.db",
        "Zend_Db_Adapter_Db2" => "ibm.db2",
        "Zend_Db_Adapter_Sqlsrv" => "microsoft.sql_server",
        _ => "other_sql",
    }
}