use crate::runtime::{jvalue::JValue, interpreter::MethodResult};

pub fn builtin_unsafe_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "registerNatives()V" => register_natives_v,
        "arrayBaseOffset0(Ljava/lang/Class;)I" | // nonsense in our impl
        "arrayIndexScale0(Ljava/lang/Class;)I" => const_0_i,
        "addressSize0()I" => address_size_i,
        "isBigEndian0()Z" |
        "unalignedAccess0()Z" => const_1_i,
        _ => panic!("Unknown jdk.internal.misc.Unsafe native: {}", name_and_desc)
    };
}

fn register_natives_v(_: Vec<JValue>) -> MethodResult{
    // no-op
    return MethodResult::Finish;
}

fn const_0_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(0));
}

fn const_1_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(1));
}

fn address_size_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(8));
}