use crate::runtime::{jvalue::JValue, interpreter::MethodResult};

pub fn builtin_string_utf8_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "isBigEndian()Z" => const_1_i,
        _ => panic!("Unknown java.lang.StringUTF16 native: {}", name_and_desc)
    };
}

fn const_1_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(1));
}