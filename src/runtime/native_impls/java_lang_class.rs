use crate::constants;
use crate::runtime::{jvalue::{JValue, JObjectData}, interpreter::{MethodResult, StackTrace}, objects, heap};

pub fn builtin_class_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "registerNatives()V" => register_natives_v,
        "getPrimitiveClass(Ljava/lang/String;)Ljava/lang/Class;" => get_primitive_class_str_class,
        "desiredAssertionStatus0(Ljava/lang/Class;)Z" => const_1_i,
        "isArray()Z" => is_array_z,
        "isPrimitive()Z" => is_primitive_z,
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
    let desc = match desc.as_str(){
        "boolean" => "Z",
        "byte" => "B",
        "short" => "S",
        "char" => "C",
        "int" => "I",
        "long" => "J",
        "float" => "F",
        "double" => "D",
        "void" => "V",
        other => panic!("Tried to create primitive class for {}!", other)
    };
    return MethodResult::FinishWithValue(heap::add_ref(objects::synthesize_class(&desc.to_string())));
}

fn const_1_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(1));
}

fn is_array_z(p: Vec<JValue>) -> MethodResult{
    return if let Some(desc) = get_desc_first(&p){
        let value = desc.starts_with("[");
        MethodResult::FinishWithValue(JValue::Int(if value { 1 } else { 0 }))
    }else{
        MethodResult::Throw(StackTrace::new(), "Could not get class descriptor in Class::isArray")
    }
}

fn is_primitive_z(p: Vec<JValue>) -> MethodResult{
    return if let Some(desc) = get_desc_first(&p){
        let value = desc.len() == 1;
        MethodResult::FinishWithValue(JValue::Int(if value { 1 } else { 0 }))
    }else{
        MethodResult::Throw(StackTrace::new(), "Could not get class descriptor in Class::isPrimitive")
    }
}

// impl

pub fn get_desc_first(p: &Vec<JValue>) -> Option<String>{
    return get_class_desc(&p[0]);
}

pub fn get_class_desc(this: &JValue) -> Option<String>{
    if let JValue::Reference(Some(this)) = this{
        let obj = this.deref();
        let data = obj.data.read();
        if let JObjectData::Fields(f) = &*data.unwrap(){
            let class_desc = f.get(constants::CLASS_DESC_FIELD_NAME);
            if let Some(desc) = class_desc{
                let as_str = objects::java_string_to_rust_string(*desc);
                return Some(as_str);
            }
        };
    }
    return None;
}