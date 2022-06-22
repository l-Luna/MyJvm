use crate::runtime::{jvalue::JValue, interpreter::MethodResult};

pub fn builtin_object_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "registerNatives()V" => register_natives_v,
        _ => panic!("Unknown java.lang.Object native: {}", name_and_desc)
    };
}

fn register_natives_v(_: Vec<JValue>) -> MethodResult{
    // no-op
    return MethodResult::Finish;
}