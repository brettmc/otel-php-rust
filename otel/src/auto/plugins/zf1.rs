use crate::{
    auto::{
        plugin::{Handler, HandlerList, HandlerSlice, HandlerCallbacks, Plugin},
    },
    config::trace_attributes,
};
use crate::{
    trace::local_root_span::get_local_root_span_context,
};
use opentelemetry::{
    KeyValue,
    trace::TraceContextExt,
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

pub struct Zf1Plugin {
    handlers: HandlerList,
}

impl Zf1Plugin {
    pub fn new() -> Self {
        Self {
            handlers: vec![
                Arc::new(Zf1RouteHandler),
                Arc::new(Zf1SendResponseHandler),
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
}

//TODO check post sendResponse and check for exceptions (anything else?)

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
        retval: &mut ZVal,
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
        ctx.span().set_attribute(KeyValue::new(trace_attributes::PHP_FRAMEWORK_NAME, "zf1"));

        // in php7, retval is optimized away (not used in Zend_Controller_Front::dispatch), so we
        // instead use the first parameter of the execute_data (which is also the request object)
        let zf1_request_zval: &mut ZVal = if retval.get_type_info() == phper::types::TypeInfo::NULL {
            let exec_data_ref = &mut *exec_data;
            exec_data_ref.get_mut_parameter(0)
        } else {
            retval
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

            //let name = format!("{} {}", request.method.as_deref().unwrap_or("GET"), route_name_str);
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
        _retval: &mut ZVal,
        _exception: Option<&mut ZObj>
    ) {
        tracing::debug!("Auto::Zf1::post (Zend_Controller_Request_Abstract::sendResponse)");

        let exec_data_ref = unsafe { &mut *exec_data };
        if let Some(this_obj) = exec_data_ref.get_this_mut() {
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
                ctx.span().set_status(opentelemetry::trace::Status::error(status_description));
            }
        }
    }
}