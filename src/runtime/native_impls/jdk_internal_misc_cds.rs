use crate::runtime::{jvalue::JValue, interpreter::MethodResult};

pub fn builtin_cds_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "initializeFromArchive(Ljava/lang/Class;)V" => no_op_v,
        "isDumpingClassList0()Z" |
        "isDumpingArchive0()Z" |
        "isSharingEnabled0()Z" => const_0_i,
        "getRandomSeedForDumping()J" => const_0_j,
        _ => panic!("Unknown jdk.internal.misc.CDS native: {}", name_and_desc)
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