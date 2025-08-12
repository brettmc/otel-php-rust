use crate::{
    auto::{
        execute_data::get_default_attributes,
        plugin::{Handler, HandlerList, HandlerSlice, HandlerCallbacks, Plugin},
    },
    config::trace_attributes,
};
use crate::{
    context::storage::{store_guard, take_guard},
    error::StringError,
    request::get_request_details,
    trace::local_root_span::{
        get_local_root_span_context,
    },
    tracer_provider,
};
use opentelemetry::{
    Context,
    KeyValue,
    trace::{
        Status,
        TraceContextExt,
        Tracer,
        TracerProvider,
    },
};
use std::{
    sync::Arc,
};
use phper::{
    alloc::ToRefOwned,
    objects::ZObj,
    values::{
        ExecuteData,
        ZVal,
    },
};

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
        let attributes = get_default_attributes(unsafe{&*exec_data});

        let span_builder = tracer.span_builder("Application::run".to_string())
            .with_attributes(attributes);
        let span = tracer.build_with_context(span_builder, &Context::current());
        let ctx = Context::current_with_span(span);
        let guard = ctx.attach();
        store_guard(exec_data, guard);
    }
    unsafe extern "C" fn post_callback(
        exec_data: *mut ExecuteData,
        _retval: &mut ZVal,
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