use crate::{
    auto::{
        plugin::{Handler, HandlerList, HandlerSlice, HandlerCallbacks, Plugin},
        utils,
    },
    config::trace_attributes,
    context::storage::{take_guard},
    error::StringError,
    request::get_request_details,
    trace::local_root_span::{
        get_local_root_span_context,
    },
    tracer_provider,
};
use opentelemetry::{
    KeyValue,
    trace::{
        SpanContext,
        SpanKind,
        Status,
        TraceContextExt,
        TracerProvider,
    },
};
use opentelemetry_semantic_conventions as SemConv;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use phper::{
    alloc::ToRefOwned,
    objects::ZObj,
    values::{
        ExecuteData,
        ZVal,
    },
};

struct ConnectionInfo {
    attributes: Vec<KeyValue>,
    span_context: SpanContext,
}

struct StatementInfo {
    attributes: Vec<KeyValue>,
    span_name: String,
    span_context: SpanContext,
}

// Storage for DB-related attributes
lazy_static! {
    static ref CONNECTION_ATTRS: Mutex<HashMap<usize, ConnectionInfo>> = Mutex::new(HashMap::new());
    static ref STATEMENT_ATTRS: Mutex<HashMap<usize, StatementInfo>> = Mutex::new(HashMap::new());
}

// Helper to get object id (pointer address)
fn get_object_id(obj: &ZObj) -> usize {
    obj as *const _ as usize
}

pub struct LaminasPlugin {
    handlers: HandlerList,
}

impl LaminasPlugin {
    pub fn new() -> Self {
        Self {
            handlers: vec![
                Arc::new(LaminasApplicationRunHandler),
                Arc::new(LaminasCompleteRequestHandler),
                Arc::new(LaminasRouteHandler),
                Arc::new(LaminasDbConnectHandler),
                Arc::new(LaminasStatementPrepareHandler),
                Arc::new(LaminasStatementExecuteHandler),
                Arc::new(LaminasConnectionExecuteHandler),
            ],
        }
    }
}

impl Plugin for LaminasPlugin {
    fn get_handlers(&self) -> &HandlerSlice {
        &self.handlers
    }
    fn get_name(&self) -> &str {
        "laminas"
    }
    fn request_shutdown(&self) {
        CONNECTION_ATTRS.lock().unwrap().clear();
        STATEMENT_ATTRS.lock().unwrap().clear();
    }
}

pub struct LaminasApplicationRunHandler;

impl Handler for LaminasApplicationRunHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some(r"Laminas\Mvc\Application".to_string()), "run".to_string()),
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

impl LaminasApplicationRunHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        match get_local_root_span_context() {
            Some(ctx) => {
                ctx.span().set_attribute(KeyValue::new(trace_attributes::PHP_FRAMEWORK_NAME, "laminas"));
            },
            None => {}
        };

        let tracer = tracer_provider::get_tracer_provider().tracer("php.otel.auto.laminas");
        let span_name = "Application::run".to_string();
        utils::start_and_activate_span(tracer, &span_name, vec![], exec_data, opentelemetry::trace::SpanKind::Internal);
    }
    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        _retval: Option<&mut ZVal>,
        _exception: Option<&mut ZObj>
    ) {
        take_guard(exec_data);
    }
}

/// Handler for Laminas\Mvc\Application::completeRequest, which is where error results are handled
pub struct LaminasCompleteRequestHandler;

impl Handler for LaminasCompleteRequestHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some(r"Laminas\Mvc\Application".to_string()), "completeRequest".to_string()),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Box::new(|exec_data| unsafe {
                Self::pre_callback(exec_data)
            })),
            post_observe: None,
        }
    }
}

impl LaminasCompleteRequestHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        //get the first argument from exec_data, which is an MvcEvent
        let exec_data_ref = unsafe { &mut *exec_data };
        let mvc_event_zval: &mut ZVal = exec_data_ref.get_mut_parameter(0);
        // see https://opentelemetry.io/docs/specs/otel/trace/exceptions/#recording-an-exception
        if let Some(mvc_event_obj) = mvc_event_zval.as_mut_z_obj() {
            let is_error = mvc_event_obj
                .call("isError", [])
                .ok()
                .and_then(|zv| zv.as_bool());
            if is_error.unwrap_or(false) {
                tracing::debug!("Auto::Laminas::pre (MvcEvent::completeRequest) - error detected");
                let context = opentelemetry::Context::current();
                let span_ref = context.span();
                //first try to get the exception param
                let exception = mvc_event_obj
                        .call("getParam", &mut [ZVal::from("exception")])
                        .ok()
                        .and_then(|mut zv| zv.as_mut_z_obj().map(|obj| obj.to_ref_owned()));
                if exception.is_some() {
                    tracing::debug!("Auto::Laminas::pre (MvcEvent::completeRequest) - exception found");
                    let attributes = crate::error::php_exception_to_attributes(&mut exception.unwrap());
                    span_ref.add_event("exception", attributes);
                    span_ref.set_status(Status::error(""));
                } else {
                    let error_str = mvc_event_obj
                        .call("getError", [])
                        .ok()
                        .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())))
                        .unwrap_or_else(|| "Unknown error".to_string());

                    let error = StringError(error_str.to_string());
                    span_ref.record_error(&error);
                    span_ref.set_status(Status::error(error_str));
                }

            }
        }
    }
}

pub struct LaminasRouteHandler;

impl Handler for LaminasRouteHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some(r"Laminas\Mvc\MvcEvent".to_string()), "setRouteMatch".to_string()),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Box::new(|exec_data| unsafe {
                Self::pre_callback(exec_data)
            })),
            post_observe: None,
        }
    }
}

impl LaminasRouteHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        tracing::debug!("Auto::Laminas::pre (MvcEvent::setRouteMatch)");
        let ctx = match get_local_root_span_context() {
            Some(ctx) => ctx,
            None => {
                tracing::debug!("Auto::Laminas::pre (MvcEvent::setRouteMatch) - no local root span/context found, skipping");
                return;
            }
        };
        let exec_data_ref = unsafe {&mut *exec_data};
        let route_match_zval: &mut ZVal = exec_data_ref.get_mut_parameter(0);
        let request = get_request_details();

        if let Some(route_match_obj) = route_match_zval.as_mut_z_obj() {
            let route_name = route_match_obj
                .call("getMatchedRouteName", [])
                .ok()
                .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())));

            let action = route_match_obj
                .call("getParam", &mut [ZVal::from("action")])
                .ok()
                .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())));

            let controller = route_match_obj
                .call("getParam", &mut [ZVal::from("controller")])
                .ok()
                .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())));

            if let Some(route_name_str) = &route_name {
                let name = format!("{} {}", request.method.as_deref().unwrap_or("GET"), route_name_str);
                tracing::debug!("Auto::Laminas::updateName (MvcEvent::setRouteMatch)");
                ctx.span().update_name(name);

                if let Some(controller) = &controller {
                    ctx.span().set_attribute(KeyValue::new(trace_attributes::PHP_FRAMEWORK_CONTROLLER_NAME, controller.clone()));
                }
                if let Some(action) = &action {
                    ctx.span().set_attribute(KeyValue::new(trace_attributes::PHP_FRAMEWORK_ACTION_NAME, action.clone()));
                }
            }
        }
    }
}

pub struct LaminasDbConnectHandler;

impl Handler for LaminasDbConnectHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some(r"Laminas\Db\Adapter\Driver\AbstractConnection".to_string()), "connect".to_string()),
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

impl LaminasDbConnectHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        utils::start_and_activate_span(
            tracer_provider::get_tracer_provider().tracer("php.otel.auto.laminas.db"),
            "connect",
            vec![],
            exec_data,
            opentelemetry::trace::SpanKind::Client
        );
        let mut attributes = vec![];

        // get connection params
        let exec_data_ref = unsafe {&mut *exec_data};
        if let Some(this_obj) = exec_data_ref.get_this_mut() {
            if let Some(zv) = this_obj.call("getConnectionParameters", []).ok() {
                if let Some(arr) = zv.as_z_arr() {
                    if let Some(database) = arr.get("database") {
                        tracing::debug!("Database: {:?}", database);
                        attributes.push(KeyValue::new(
                            SemConv::trace::DB_NAMESPACE,
                            database.as_z_str()
                                .and_then(|s| s.to_str().ok())
                                .unwrap_or_default()
                                .to_string()
                        ));
                    }
                    let system = arr.get("driver")
                        .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok()))
                        .map(|driver| map_laminas_driver_to_semconv(driver))
                        .unwrap_or_default();
                    attributes.push(KeyValue::new(SemConv::trace::DB_SYSTEM_NAME, system.to_string()));
                    //add attributes to current span
                    opentelemetry::Context::current().span().set_attributes(attributes.clone());
                }
            }
            let id = get_object_id(this_obj);
            CONNECTION_ATTRS.lock().unwrap().insert(id, ConnectionInfo{
                attributes: attributes.clone(),
                span_context: opentelemetry::Context::current().span().span_context().clone(),
            });
            tracing::debug!("Auto::Laminas::pre (connect) - storing connection attributes: {:?}, id={}", attributes, id);
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

pub struct LaminasStatementPrepareHandler;

impl Handler for LaminasStatementPrepareHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some(r"Laminas\Db\Adapter\Driver\StatementInterface".to_string()), "prepare".to_string()),
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

impl LaminasStatementPrepareHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        utils::start_and_activate_span(
            tracer_provider::get_tracer_provider().tracer("php.otel.auto.laminas.db"),
            "prepare",
            vec![],
            exec_data,
            opentelemetry::trace::SpanKind::Client
        );
    }
    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        _retval: Option<&mut ZVal>,
        exception: Option<&mut ZObj>
    ) {
        tracing::debug!("Auto::Laminas::post (Statement::prepare) - post_callback called");
        let _guard = take_guard(exec_data);

        if let Some(exception) = exception {
            utils::record_exception(&opentelemetry::Context::current(), exception);
        }

        let mut prepare_span_name = "prepare".to_string();
        let mut prepare_attributes = vec![];

        // Get the first parameter as a string, if present
        let sql_from_param = {
            let exec_data_ref = unsafe { &mut *exec_data };
            let sql_zval = exec_data_ref.get_mut_parameter(0);
            if sql_zval.get_type_info() != phper::types::TypeInfo::NULL {
                sql_zval.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned()))
            } else {
                None
            }
        };

        // Now get this_obj and use it for everything else
        let exec_data_ref = unsafe { &mut *exec_data };
        if let Some(this_obj) = exec_data_ref.get_this_mut() {
            let mut execute_span_name = "OTHER".to_string();
            let sql = sql_from_param.or_else(|| {
                this_obj.call("getSql", [])
                    .ok()
                    .and_then(|zv| zv.as_z_str().and_then(|s| s.to_str().ok().map(|s| s.to_owned())))
            });

            let mut attributes = vec![];
            tracing::debug!("Auto::Laminas::post (Statement::prepare) - sql: {:?}", sql);
            if let Some(sql_str) = sql {
                attributes.push(KeyValue::new(SemConv::trace::DB_QUERY_TEXT, sql_str.clone()));
                prepare_attributes.push(KeyValue::new(SemConv::trace::DB_QUERY_TEXT, sql_str.clone()));
                let sql_name = utils::extract_span_name_from_sql(&sql_str)
                    .unwrap_or_else(|| "OTHER".to_string());
                execute_span_name = sql_name.clone();
                prepare_span_name = format!("prepare {}", sql_name.clone());
            }

            //look up connection attributes, add to prepare and roll up to statement attributes
            let driver_zval: &mut ZVal = this_obj.get_mut_property("driver");
            if let Some(driver_obj) = driver_zval.as_mut_z_obj() {
                if let Ok(connection_zval) = driver_obj.call("getConnection", []) {
                    if let Some(connection_obj) = connection_zval.as_z_obj() {
                        let connection_id = get_object_id(connection_obj);
                        tracing::debug!("Auto::Laminas::post (Statement::prepare) - found driver connection id={}", connection_id);
                        let connection_attrs_guard = CONNECTION_ATTRS.lock().unwrap();
                        let connection_info = connection_attrs_guard.get(&connection_id);
                        if let Some(connection_info) = connection_info {
                            attributes.extend_from_slice(&connection_info.attributes);
                            prepare_attributes.extend_from_slice(&connection_info.attributes);
                            //add span context as a link to current span
                            opentelemetry::Context::current().span().add_link(
                                connection_info.span_context.clone(),
                                vec![]
                            );
                        }
                    }
                }
            }

            let id = get_object_id(this_obj);
            tracing::debug!("Auto::Laminas::post (Statement::prepare) - storing statement attributes for statement id: {}", id);
            STATEMENT_ATTRS.lock().unwrap().insert(id, StatementInfo{
                attributes,
                span_name: execute_span_name,
                span_context: opentelemetry::Context::current().span().span_context().clone(),
            });
        }
        let ctx = opentelemetry::Context::current();
        let span = ctx.span();
        span.update_name(prepare_span_name);
        span.set_attributes(prepare_attributes);
    }
}

pub struct LaminasStatementExecuteHandler;

impl Handler for LaminasStatementExecuteHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some(r"Laminas\Db\Adapter\Driver\StatementInterface".to_string()), "execute".to_string()),
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

impl LaminasStatementExecuteHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        tracing::debug!("Auto::Laminas::pre (Statement::execute) - pre_callback called");
        let tracer = tracer_provider::get_tracer_provider().tracer("php.otel.auto.laminas.db");
        let span_name = "Statement::execute".to_string();
        utils::start_and_activate_span(tracer, &span_name, vec![], exec_data, SpanKind::Client);
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
        let exec_data_ref = unsafe {&mut *exec_data};
        if let Some(this_obj) = exec_data_ref.get_this_mut() {
            let statement_id = get_object_id(this_obj);
            tracing::debug!("Auto::Laminas::pre (Statement::execute) - found this object: id={}", statement_id);
            if let Some(info) = STATEMENT_ATTRS.lock().unwrap().get(&statement_id) {
                tracing::debug!("Auto::Laminas::pre (Statement::execute) - found statement attributes: {:?}", info.attributes);
                let mut attributes = vec![];
                attributes.extend_from_slice(&info.attributes);
                let span_name = info.span_name.clone();
                let ctx = opentelemetry::Context::current();
                let span = ctx.span();
                span.set_attributes(attributes);
                span.update_name(span_name);
                span.add_link(
                    info.span_context.clone(),
                    vec![]
                );
            }
        }
    }
}

pub struct LaminasConnectionExecuteHandler;

impl Handler for LaminasConnectionExecuteHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some(r"Laminas\Db\Adapter\Driver\ConnectionInterface".to_string()), "execute".to_string()),
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

impl LaminasConnectionExecuteHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        tracing::debug!("Auto::Laminas::pre (Connection::execute) - pre_callback called");
        let tracer = tracer_provider::get_tracer_provider().tracer("php.otel.auto.laminas.db");
        let exec_data_ref = unsafe {&mut *exec_data};
        let mut attributes = vec![];
        //sql param
        let sql_zval: &mut ZVal = exec_data_ref.get_mut_parameter(0);
        let sql_str = sql_zval.as_z_str().and_then(|s| s.to_str().ok()).unwrap_or_default();

        attributes.push(KeyValue::new(SemConv::trace::DB_QUERY_TEXT, sql_str));
        let span_name = utils::extract_span_name_from_sql(sql_str)
            .unwrap_or_else(|| "OTHER".to_string());

        utils::start_and_activate_span(tracer, &span_name, attributes, exec_data, opentelemetry::trace::SpanKind::Client);
    }

    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        _retval: Option<&mut ZVal>,
        exception: Option<&mut ZObj>
    ) {
        tracing::debug!("Auto::Laminas::post (Connection::execute) - post_callback called");
        let _guard = take_guard(exec_data);
        if let Some(exception) = exception {
            utils::record_exception(&opentelemetry::Context::current(), exception);
        }
    }
}

fn map_laminas_driver_to_semconv(driver: &str) -> &str {
    match driver.to_lowercase().as_str() {
        "mysqli" | "pdo_mysql" => "mysql",
        "pgsql" | "pdo_pgsql" => "postgresql",
        "sqlite" | "pdo_sqlite" => "sqlite",
        "oci8" | "pdo_oci" => "oracle",
        "sqlsrv" | "pdo_sqlsrv" => "mssql",
        "ibm_db2" => "db2",
        "pdo_firebird" => "firebird",
        "pdo_dblib" => "mssql",
        "pdo_odbc" => "odbc",
        _ => "other_sql",
    }
}