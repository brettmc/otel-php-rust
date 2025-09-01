use crate::auto::plugin::hook::add_hook;
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

                add_hook(class.clone(), function.clone(), pre, post);

                Ok::<_, phper::Error>(true)
            }
        )
        .argument(Argument::new("class").optional().with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("function").with_type_hint(ArgumentTypeHint::String))
        .argument(Argument::new("pre").optional().with_type_hint(ArgumentTypeHint::Callable))
        .argument(Argument::new("post").optional().with_type_hint(ArgumentTypeHint::Callable))
        .return_type(ReturnType::new(ReturnTypeHint::Bool))
    ;
}

