use crate::auto::{
    plugin::{Handler, HandlerCallbacks, Plugin},
};
use crate::trace::local_root_span::get_local_root_span;
use crate::context::storage;
use opentelemetry::trace::TraceContextExt;
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
    /*
        $rm = $e->getRouteMatch();
        $method = $e->getRequest()->getMethod();
        $routeName = $rm->getMatchedRouteName();

        $config = $e->getApplication()->getServiceManager()->get('config');
        $routeConfig = $config['router']['routes'][$routeName] ?? null;

        $route = $routeConfig['options']['route'] ?? $routeName;
        LocalRootSpan::current()->updateName(sprintf('%s %s', $method, $route));
     */
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        let exec_data_ref = &mut *exec_data;
        let mvc_event_zval: &mut ZVal = exec_data_ref.get_mut_parameter(0);

        //TODO add more SemConv attributes...
        if let Some(mvc_event_obj) = mvc_event_zval.as_mut_z_obj() {
            if let Ok(mut route_match_zval) = mvc_event_obj.call("getRouteMatch", []) {
                if let Some(route_match_obj) = route_match_zval.as_mut_z_obj() {
                    if let Ok(route_name_zval) = route_match_obj.call("getMatchedRouteName", []) {
                        if let Some(route_name_str) = route_name_zval.as_z_str().and_then(|s| s.to_str().ok()) {
                            let name = format!("<method> {}", route_name_str); //todo method
                            //todo this is an ID, need to fetch from storage from span.rs
                            let instance_id = get_local_root_span().unwrap_or(0);
                            if let Some(ctx) = storage::get_context_instance(instance_id as u64) {
                                tracing::debug!("Auto::Laminas::updateName (post MvcEvent::setRouteMatch)");
                                ctx.span().update_name(name);
                            }
                        }
                    }
                }
            }
            //todo get method
        }
    }
}
