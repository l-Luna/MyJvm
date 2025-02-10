use crate::runtime::interpreter::MethodResult;
use crate::runtime::jvalue::JValue;

pub fn builtin_sma_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult {
    return match name_and_desc {
        "registerNatives()V" => no_op_v,
        _ => panic!("Unknown jdk.internal.misc.ScopedMemoryAccess native: {}", name_and_desc)
    }
}

fn no_op_v(_: Vec<JValue>) -> MethodResult{
    return MethodResult::Finish;
}