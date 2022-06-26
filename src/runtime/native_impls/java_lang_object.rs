use runtime::jvalue::JValue::Reference;
use StackTrace;
use crate::runtime::{jvalue::JValue, interpreter::MethodResult};

pub fn builtin_object_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "registerNatives()V" => register_natives_v,
        "hashCode()I" => hash_code_i,
        _ => panic!("Unknown java.lang.Object native: {}", name_and_desc)
    };
}

fn register_natives_v(_: Vec<JValue>) -> MethodResult{
    // no-op
    return MethodResult::Finish;
}

fn hash_code_i(args: Vec<JValue>) -> MethodResult{
    let this = args[0];
    return if let Reference(Some(this)) = this{
        MethodResult::FinishWithValue(JValue::Int(this.deref().identity_hash))
    }else{
        MethodResult::Throw(StackTrace::new(), "NPE in Object::hashCode")
    }
}