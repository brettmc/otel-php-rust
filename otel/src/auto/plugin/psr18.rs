use crate::{
    auto::{
        plugin::{Handler, HandlerList, HandlerSlice, HandlerCallbacks, Plugin},
        utils::{start_and_activate_span, record_exception},
    },
    context::storage::{take_guard},
    trace::tracer_provider,
};
use opentelemetry::{
    KeyValue,
    Context,
    global,
    trace::{
        SpanKind,
        TraceContextExt,
        TracerProvider,
    },
};
use opentelemetry_semantic_conventions as SemConv;
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
    handlers: HandlerList,
}

impl Psr18Plugin {
    pub fn new() -> Self {
        Self {
            handlers: vec![
                Arc::new(Psr18SendRequestHandler),
            ],
        }
    }
}

impl Plugin for Psr18Plugin {
    fn get_handlers(&self) -> &HandlerSlice {
        &self.handlers
    }
    fn get_name(&self) -> &str {
        "psr-18"
    }
}

pub struct Psr18SendRequestHandler;

impl Handler for Psr18SendRequestHandler {
    fn get_targets(&self) -> Vec<(Option<String>, String)> {
        vec![
            (Some(r"Psr\Http\Client\ClientInterface".to_string()), "sendRequest".to_string()),
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

impl Psr18SendRequestHandler {
    unsafe extern "C" fn pre_callback(exec_data: *mut ExecuteData) {
        let tracer = tracer_provider::get_tracer_provider().tracer("php.otel.auto.psr18");
        let mut name = "psr18.request".to_string();

        let exec_data_ref = unsafe {&mut *exec_data};
        let mut attributes = vec![];
        let request_zval: &mut ZVal = exec_data_ref.get_mut_parameter(0);

        //TODO add more SemConv attributes...
        if let Some(request_obj) = request_zval.as_mut_z_obj() {
            if let Ok(mut uri_zval) = request_obj.call("getUri", []) {
                if let Some(uri_obj) = uri_zval.as_mut_z_obj() {
                    if let Ok(uri_str_zval) = uri_obj.call("__toString", []) {
                        if let Some(uri_str) = uri_str_zval.as_z_str().and_then(|s| s.to_str().ok()) {
                            attributes.push(KeyValue::new(SemConv::trace::URL_FULL, uri_str.to_owned()));
                        }
                    }
                    uri_obj.call("getScheme", [])
                        .ok()
                        .and_then(|scheme_zval| scheme_zval.as_z_str()?.to_str().ok().map(|s| s.to_owned()))
                        .map(|scheme| attributes.push(KeyValue::new(SemConv::trace::URL_SCHEME, scheme)));
                    uri_obj.call("getPath", [])
                        .ok()
                        .and_then(|path_zval| path_zval.as_z_str()?.to_str().ok().map(|s| s.to_owned()))
                        .map(|path| attributes.push(KeyValue::new(SemConv::trace::URL_PATH, path)));
                    uri_obj.call("getHost", [])
                        .ok()
                        .and_then(|host_zval| host_zval.as_z_str()?.to_str().ok().map(|s| s.to_owned()))
                        .map(|host| attributes.push(KeyValue::new(SemConv::trace::SERVER_ADDRESS, host)));
                    uri_obj.call("getPort", [])
                        .ok()
                        .and_then(|port_zval| port_zval.as_long())
                        .map(|port| attributes.push(KeyValue::new(SemConv::trace::SERVER_PORT, port)));
                }
            }
            if let Ok(method_zval) = request_obj.call("getMethod", []) {
                if let Some(method_str) = method_zval.as_z_str().and_then(|s| s.to_str().ok()) {
                    attributes.push(KeyValue::new(SemConv::trace::HTTP_REQUEST_METHOD, method_str.to_owned()));
                    name = method_str.to_string();
                }
            }
        }

        start_and_activate_span(tracer, &name, attributes, exec_data, SpanKind::Client);

        //now inject the trace context into the request headers, using the span we just started
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
        exec_data: *mut ExecuteData,
        _retval: Option<&mut ZVal>,
        exception: Option<&mut ZObj>
    ) {
        let _guard = take_guard(exec_data);
        //get the current span
        let context = Context::current();
        let span_ref = context.span();
        if let Some(exception) = exception {
            record_exception(&opentelemetry::Context::current(), exception);
        }

        let exec_data_ref = unsafe { &mut *exec_data };
        match exec_data_ref.get_return_value_mut() {
            Some(retval) => {
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
            None => {
                tracing::warn!("Psr18Handler: no return value found");
            }
        }
    }
}
