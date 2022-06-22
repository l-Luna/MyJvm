use crate::runtime::{jvalue::JValue, interpreter::MethodResult, objects, heap};

pub fn builtin_class_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "registerNatives()V" => register_natives_v,
        "getPrimitiveClass(Ljava/lang/String;)Ljava/lang/Class;" => get_primitive_class_str_class,
        _ => panic!("Unknown java.lang.Class native: {}", name_and_desc)
    };
}

fn register_natives_v(_: Vec<JValue>) -> MethodResult{
    // no-op
    return MethodResult::Finish;
}

fn get_primitive_class_str_class(params: Vec<JValue>) -> MethodResult{
    let desc = params[0];
    let desc = objects::java_string_to_rust_string(desc);
    //return MethodResult::FinishWithValue(heap::bt_class_by_desc(desc).unwrap());
    return MethodResult::FinishWithValue(JValue::Reference(None));
}