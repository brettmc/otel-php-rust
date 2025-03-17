use crate::trace::plugin::{Handler, HandlerCallbacks, Plugin, SpanDetails};
use opentelemetry::{
    KeyValue,
    Context,
    global,
    trace::{SpanKind, SpanRef},
};
use std::{
    sync::Arc,
    collections::HashMap,
};
use phper::{
    objects::ZObj,
    values::{
        ExecuteData,
        ZVal,
    },
};

pub struct Psr18Plugin {
    handlers: Vec<Arc<dyn Handler + Send + Sync>>,
}

impl Psr18Plugin {
    pub fn new() -> Self {
        Self {
            handlers: vec![
                Arc::new(Psr18Handler),
            ],
        }
    }
}

impl Plugin for Psr18Plugin {
    fn is_enabled(&self) -> bool {
        true
    }
    fn get_handlers(&self) -> Vec<Arc<dyn Handler + Send + Sync>> {
        self.handlers.clone()
    }
    fn get_name(&self) -> &str {
        "psr-18"
    }
}

pub struct Psr18Handler;

impl Handler for Psr18Handler {
    fn get_functions(&self) -> Vec<String> {
        vec![]
    }
    fn get_interfaces(&self) -> Vec<String> {
        vec![
            r"Psr\Http\Client\ClientInterface::sendRequest".to_string(),
        ]
    }
    fn get_callbacks(&self) -> HandlerCallbacks {
        HandlerCallbacks {
            pre_observe: Some(Self::pre_callback),
            post_observe: Some(Self::post_callback),
        }
    }
}

impl Psr18Handler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData, span_details: &mut SpanDetails) {
        //TODO this uses parent span (if there is one) as the traceparent header, since this span is not
        //     active yet. Should this handler create its own span??
        let exec_data_ref = &mut *exec_data;
        let request_zval: &mut ZVal = exec_data_ref.get_mut_parameter(0);
        span_details.update_name("GET");
        span_details.set_kind(SpanKind::Client);

        let mut carrier = HashMap::new();
        global::get_text_map_propagator(|prop| prop.inject_context(&Context::current(), &mut carrier));

        let mut modified_request = request_zval.clone();
        for (key, value) in carrier {
            if let Some(updated_request) = modified_request
            .as_mut_z_obj()
            .and_then(|obj| obj.call("withHeader", &mut [ZVal::from(key.clone()), ZVal::from(value)]).ok()) 
            {
                modified_request = updated_request;
            } else {
                tracing::warn!("Psr18Handler: failed to inject trace header: {}", key);
            }
        }
        *request_zval = modified_request;
    }

    unsafe extern "C" fn post_callback(
        _exec_data: *mut ExecuteData,
        span_ref: &SpanRef,
        retval: &mut ZVal,
    ) {
        if !retval.get_type_info().is_object() {
            tracing::warn!("Psr18Handler: return value is not an object");
            return;
        }

        let response_obj: &mut ZObj = match retval.as_mut_z_obj() {
            Some(obj) => obj,
            None => {
                tracing::warn!("Psr18Handler: failed to convert return value to object");
                return;
            }
        };

        let status_code_zval = match response_obj.call("getStatusCode", &mut []) {
            Ok(zval) => zval,
            Err(_) => {
                tracing::warn!("Psr18Handler: failed to call getStatusCode()");
                return;
            }
        };

        let status_code = match status_code_zval.as_long() {
            Some(code) => code,
            None => {
                tracing::warn!("Psr18Handler: getStatusCode() did not return an integer");
                return;
            }
        };

        span_ref.set_attribute(KeyValue::new("http.response.status_code", status_code));
    }
}
