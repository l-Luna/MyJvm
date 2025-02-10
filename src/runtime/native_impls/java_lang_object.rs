use crate::runtime::{heap, objects};
use crate::runtime::{jvalue::JValue, interpreter::{MethodResult, StackTrace}};

pub fn builtin_object_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "registerNatives()V" |
        "notifyAll()V" => no_op_v,
        "hashCode()I" => hash_code_i,
        "getClass()Ljava/lang/Class;" => get_class,
        _ => panic!("Unknown java.lang.Object native: {}", name_and_desc)
    };
}

fn no_op_v(_: Vec<JValue>) -> MethodResult{
    // no-op
    return MethodResult::Finish;
}

fn hash_code_i(args: Vec<JValue>) -> MethodResult{
    let this = args[0];
    return if let JValue::Reference(Some(this)) = this{
        MethodResult::FinishWithValue(JValue::Int(this.deref().identity_hash))
    }else{
        MethodResult::Throw(StackTrace::new(), "NPE in Object::hashCode")
    }
}

fn get_class(args: Vec<JValue>) -> MethodResult{
    let this = args[0];
    return if let JValue::Reference(Some(this)) = this{
        MethodResult::FinishWithValue(heap::add_ref(objects::synthesize_class(&this.deref().class.descriptor)))
    }else{
        MethodResult::Throw(StackTrace::new(), "NPE in Object::getClass")
    }
}