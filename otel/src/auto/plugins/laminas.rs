use crate::auto::{
    plugin::{Handler, HandlerList, HandlerSlice, HandlerCallbacks, Plugin},
};
use crate::{
    request::get_request_details,
    trace::local_root_span::{
        get_local_root_span_context,
    },
};
use opentelemetry::{
    KeyValue,
    trace::TraceContextExt,
};
use std::{
    sync::Arc,
};
use phper::{
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

pub struct LaminasRouteHandler;

impl Handler for LaminasRouteHandler {
    fn get_functions(&self) -> Vec<String> {
        vec![]
    }
    fn get_interfaces(&self) -> Vec<String> {
        vec![
            r"Laminas\Mvc\MvcEvent::setRouteMatch".to_string(),
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
        let exec_data_ref = &mut *exec_data;
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
                ctx.span().set_attribute(KeyValue::new("php.framework.name", "laminas"));
                if let Some(controller) = &controller {
                    ctx.span().set_attribute(KeyValue::new("php.framework.controller.name", controller.clone()));
                }
                if let Some(action) = &action {
                    ctx.span().set_attribute(KeyValue::new("php.framework.action.name", action.clone()));
                }
            }
        }
    }
}
