// methods for building java objects (e.g. string constants)

use std::collections::HashMap;
use std::sync::Arc;
use crate::runtime::{jvalue::{JObject, JObjectData, JValue}, class::ClassRef, classes, heap};

use crate::constants;

pub fn create_new(of: ClassRef) -> JValue{
    let mut fields = HashMap::with_capacity(of.instance_fields.len());
    for f in &of.instance_fields{
        fields.insert(f.name.clone(), JValue::default_value_for(&f.type_class.descriptor()));
    }
    return heap::add_ref(JObject::new(of, JObjectData::Fields(fields)));
}

pub fn create_new_array(of: ClassRef, length: usize) -> JValue{
    let mut elements = Vec::with_capacity(length);
    for _ in 0..length{
        elements.push(JValue::default_value_for(&of.descriptor));
    }
    return heap::add_ref(JObject::new(of, JObjectData::Array(length, elements)));
}

pub fn create_new_array_of(of: ClassRef, values: Vec<JValue>) -> JValue{
    return heap::add_ref(JObject::new(of, JObjectData::Array(values.len(), values)));
}

/// Create a new Java string object with the given text.
pub fn synthesize_string(string: &String) -> JObject{
    let mut fields = HashMap::with_capacity(4);
    fields.insert("value".to_owned(), array_of(wrap_bytes(as_utf16(string))));
    fields.insert("coder".to_owned(), JValue::Int(1)); // always UTF16
    fields.insert("hash".to_owned(), JValue::Int(0)); // let java figure it out; these are default values
    fields.insert("hashIsZero".to_owned(), JValue::Int(0));
    return JObject::new(string_class(), JObjectData::Fields(fields));
}

// TODO!: cache class objects for ==
/// Create a new Java class object with the given descriptor.
/// The descriptor is stored in an undeclared field `JVM_DESCRIPTOR`.
pub fn synthesize_class(descriptor: &String) -> JObject{
    let mut fields = HashMap::with_capacity(7 + 1);
    fields.insert(constants::CLASS_DESC_FIELD_NAME.to_owned(), heap::add_ref(synthesize_string(descriptor)));
    return JObject::new(class_class(), JObjectData::Fields(fields));
}

pub fn java_string_to_rust_string(jstring: JValue) -> String{
    if let JValue::Reference(Some(r)) = jstring{
        let obj = r.deref();
        if let JObjectData::Fields(f) = &*obj.data.read().unwrap(){
            let value = f["value"];
            if let JValue::Reference(Some(r)) = value{
                let array = r.deref();
                if let JObjectData::Array(_, v) = &*array.data.read().unwrap(){
                    let bytes = unwrap_bytes(v);
                    let bytes: Vec<u16> = bytes
                        .chunks_exact(2)
                        .into_iter()
                        .map(|a| u16::from_be_bytes([a[0], a[1]]))
                        .collect();
                    return String::from_utf16(&bytes).unwrap();
                };
            }
        };
    }
    panic!("Tried to convert a non-java-string to a rust string!");
}

// implementation
// all panic rather than erroring

fn wrap_bytes(ints: Vec<u8>) -> Vec<JValue>{
    return ints.iter().map(|i| JValue::Int(*i as i32)).collect();
}

fn unwrap_bytes(ints: &Vec<JValue>) -> Vec<u8>{
    let mut ret = Vec::with_capacity(ints.len());
    for v in ints{
        if let JValue::Int(i) = v{
            ret.push(*i as u8);
        }else{
            panic!("Tried to call objects::unwrap_bytes on a vec with non-ints!");
        }
    }
    return ret;
}

fn as_utf16(string: &str) -> Vec<u8>{
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
    return heap::add_ref(JObject::new(class, JObjectData::Array(objects.len(), objects)));
}

pub fn force_init_class(desc: &str) -> ClassRef{
    return heap::get_or_create_bt_class(desc.to_string())
        .expect(format!("Could not parse {}!", desc).as_str())
        .ensure_initialized()
        .expect(format!("Could not link {}!", desc).as_str());
}

pub fn string_class() -> ClassRef{
    return force_init_class("Ljava/lang/String;");
}

pub fn class_class() -> ClassRef{
    return force_init_class("Ljava/lang/Class;");
}