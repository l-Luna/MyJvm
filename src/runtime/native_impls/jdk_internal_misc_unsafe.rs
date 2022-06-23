use crate::runtime::{jvalue::JValue, interpreter::MethodResult};

pub fn builtin_unsafe_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "registerNatives()V" |
        "storeFence()V" => no_op_v,
        "arrayBaseOffset0(Ljava/lang/Class;)I" | // nonsense in our impl
        "arrayIndexScale0(Ljava/lang/Class;)I" => const_0_i,
        "addressSize0()I" => address_size_i,
        "isBigEndian0()Z" |
        "unalignedAccess0()Z" => const_1_i,
        "objectFieldOffset1(Ljava/lang/Class;Ljava/lang/String;)J" => const_0_j,
        _ => panic!("Unknown jdk.internal.misc.Unsafe native: {}", name_and_desc)
    };
}

fn no_op_v(_: Vec<JValue>) -> MethodResult{
    return MethodResult::Finish;
}

fn const_0_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(0));
}

fn const_0_j(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Long(0));
}

fn const_1_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(1));
}

fn address_size_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(8));
}