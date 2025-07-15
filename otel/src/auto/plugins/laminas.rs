use crate::auto::{
    plugin::{Handler, HandlerCallbacks, Plugin},
};
use crate::{
    request::get_request_details,
    trace::local_root_span::get_local_root_span,
};
use crate::context::storage;
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
    handlers: Vec<Arc<dyn Handler + Send + Sync>>,
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
    fn is_enabled(&self) -> bool {
        true
    }
    fn get_handlers(&self) -> Vec<Arc<dyn Handler + Send + Sync>> {
        self.handlers.clone()
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
            pre_observe: Some(Self::pre_callback),
            post_observe: None,
        }
    }
}

impl LaminasRouteHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        tracing::debug!("Auto::Laminas::pre (MvcEvent::setRouteMatch)");
        let instance_id = get_local_root_span().unwrap_or(0);
        if instance_id == 0 {
            tracing::debug!("Auto::Laminas::pre (MvcEvent::setRouteMatch) - no local root span found, skipping");
            return;
        }
        let ctx = match storage::get_context_instance(instance_id as u64) {
            Some(ctx) => ctx,
            None => {
                tracing::warn!("Auto::Laminas::pre (MvcEvent::setRouteMatch) - no context found for instance id {}", instance_id);
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
