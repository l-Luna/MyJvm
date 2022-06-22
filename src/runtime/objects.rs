// methods for building java objects (e.g. string constants)

use std::collections::HashMap;
use std::sync::Arc;
use runtime::{class::ClassRef, classes, heap};
use runtime::jvalue::{JObject, JObjectData, JValue};

pub fn synthesize_string(string: &String) -> JObject{
    let mut fields = HashMap::with_capacity(4);
    fields.insert("value".to_owned(), array_of(wrap_ints(as_utf16(string))));
    fields.insert("coder".to_owned(), JValue::Int(1)); // always UTF16
    fields.insert("hash".to_owned(), JValue::Int(1)); // let java figure it out; these are default values, made explicit
    fields.insert("hashIsZero".to_owned(), JValue::Int(1));
    return JObject{
        class: string_class(),
        data: JObjectData::Fields(fields)
    };
}

// implementation
// all panic rather than erroring

fn wrap_ints(ints: Vec<u8>) -> Vec<JValue>{
    return ints.iter().map(|i| JValue::Int(*i as i32)).collect();
}

fn as_utf16(string: &String) -> Vec<u8>{
    let mut ret = Vec::with_capacity(string.len() * 2);
    for i in string.encode_utf16(){
        for b in i.to_be_bytes(){
            ret.push(b);
        }
    }
    return ret;
}

fn array_of(objects: Vec<JValue>) -> JValue{
    let class = if objects.len() == 0{
        heap::bt_class_by_desc("Ljava/lang/Object;".to_owned()).unwrap()
    }else{
        objects[0].class()
    };
    let class = Arc::new(classes::array_class(&class));
    return heap::add_ref(JObject{
        class,
        data: JObjectData::Array(objects)
    });
}

fn string_class() -> ClassRef{
    return heap::get_or_create_bt_class("Ljava/lang/String;".to_string())
        .expect("Could not parse java.lang.String!")
        .ensure_loaded()
        .expect("Could not link java.lang.String!");
}