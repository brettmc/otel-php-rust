use crate::trace::plugin::{Handler, HandlerCallbacks, Plugin, SpanDetails};
use opentelemetry::{
    KeyValue,
    Context,
    global,
    trace::{SpanKind, SpanRef},
};
use opentelemetry_semantic_conventions as SemConv;
use std::{
    sync::Arc,
    collections::HashMap,
};
use phper::{
    alloc::ToRefOwned,
    errors::ThrowObject,
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
        span_details.set_kind(SpanKind::Client);

        //TODO add more SemConv attributes...
        if let Some(request_obj) = request_zval.as_mut_z_obj() {
            if let Ok(mut uri_zval) = request_obj.call("getUri", []) {
                if let Some(uri_obj) = uri_zval.as_mut_z_obj() {
                    if let Ok(uri_str_zval) = uri_obj.call("__toString", []) {
                        if let Some(uri_str) = uri_str_zval.as_z_str().and_then(|s| s.to_str().ok()) {
                            span_details.add_attribute(String::from(SemConv::trace::URL_FULL), uri_str.to_owned());
                        }
                    }
                }
            }
            if let Ok(method_zval) = request_obj.call("getMethod", []) {
                if let Some(method_str) = method_zval.as_z_str().and_then(|s| s.to_str().ok()) {
                    span_details.add_attribute(String::from(SemConv::trace::HTTP_REQUEST_METHOD), method_str.to_owned());
                    span_details.update_name(method_str);
                }
            }
        }

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
        exception: Option<&mut ZObj>
    ) {
        if let Some(exception) = exception {
            if let Ok(throwable) = ThrowObject::new(exception.to_ref_owned()) {
                span_ref.record_error(&throwable);
            }
        }

        if !retval.get_type_info().is_object() {
            // no return value, nothing else to do
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

        span_ref.set_attribute(KeyValue::new(SemConv::trace::HTTP_RESPONSE_STATUS_CODE, status_code));
    }
}
