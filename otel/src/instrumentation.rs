use crate::auto::plugin::hook::{add_hook, RegisteredHook};
use phper::functions::{Argument, ReturnType};
use phper::modules::Module;
use phper::types::{ArgumentTypeHint, ReturnTypeHint};

pub fn register_instrumentation_functions(module: &mut Module) {
    module.add_function(
        r"OpenTelemetry\Instrumentation\hook",
            |args| -> Result<bool, phper::Error> {
                let class = args.get(0)
                    .and_then(|arg| arg.as_z_str())
                    .and_then(|s| s.to_str().ok())
                    .map(|s| s.to_string());

                let function = match args.get(1)
                    .and_then(|arg| arg.as_z_str())
                    .and_then(|s| s.to_str().ok()) {
                        Some(s) => s.to_string(),
                        None => return Err(phper::Error::boxed("function argument must be a string")),
                    };

                let pre = args.get(2).cloned();
                let post = args.get(3).cloned();

                /*if let Some(mut pre_hook) = pre {
                    if let Some(zobj) = pre_hook.as_mut_z_obj() {
                        zobj.call("__invoke", [])?;
                    } else {
                        return Err(phper::Error::boxed("pre hook must be an object"));
                    }
                } else {
                    return Err(phper::Error::boxed("pre hook is required"));
                }*/

                let hook = RegisteredHook {
                    class: class.clone(),
                    function: function.clone(),
                    pre,
                    post,
                };
                add_hook(hook);

                Ok::<_, phper::Error>(true)
            }
        )
        .argument(Argument::new("class"))
        .argument(Argument::new("function"))
        .argument(Argument::new("pre").optional())
        .argument(Argument::new("post").optional())
        //.return_type(ReturnType::new(ReturnTypeHint::Bool))
    ;
}

