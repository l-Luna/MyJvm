use runtime::interpreter::MethodResult;
use runtime::jvalue::JValue;

pub fn builtin_system_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "registerNatives()V" => register_natives_v,
        _ => panic!("Unknown java.lang.System native: {}", name_and_desc)
    };
}

fn register_natives_v(_: Vec<JValue>) -> MethodResult{
    // TODO: set System.out and System.in? after they're set to null?
    return MethodResult::Finish;
}