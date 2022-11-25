use crate::runtime::{jvalue::JValue, interpreter::MethodResult};

pub fn builtin_access_controller_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "getStackAccessControlContext()Ljava/security/AccessControlContext;" => const_null_acc,
        _ => panic!("Unknown java.security.AccessController native: {}", name_and_desc)
    };
}

fn const_null_acc(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Reference(None));
}