use crate::runtime::{jvalue::JValue, interpreter::MethodResult};

pub fn builtin_throwable_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "fillInStackTrace(I)Ljava/lang/Throwable;" => noop,
        _ => panic!("Unknown java.lang.Throwable native: {}", name_and_desc)
    };
}

fn noop(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Reference(None));
}