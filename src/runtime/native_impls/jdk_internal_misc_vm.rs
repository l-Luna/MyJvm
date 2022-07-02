use runtime::interpreter::MethodResult;
use runtime::jvalue::JValue;

pub fn builtin_vm_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "initialize()V" => initialize_v,
        _ => panic!("Unknown jdk.internal.misc.VM native: {}", name_and_desc)
    };
}

fn initialize_v(_: Vec<JValue>) -> MethodResult{
    return MethodResult::Finish;
}