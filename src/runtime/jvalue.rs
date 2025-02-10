use std::{collections::HashMap, sync::RwLock};
use rand::random;
use crate::runtime::heap;

use super::{heap::JRef, class::ClassRef};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JValue{
    Int(i32), // and other int-likes
    Long(i64),
    Float(f32),
    Double(f64),

    Second,

    Reference(Option<JRef>) // None = null
}

#[derive(Debug)]
pub struct JObject{
    pub class: ClassRef,
    pub identity_hash: i32, // just a random number
    pub data: RwLock<JObjectData>
}

impl JObject{
    pub fn new(class: ClassRef, data: JObjectData) -> JObject{
        return JObject{
            class,
            identity_hash: random(),
            data: RwLock::new(data)
        };
    }

    /// Returns the descriptor of the type of this object.
    pub fn descriptor(&self) -> String{
        return match &*self.data.read().unwrap(){
            JObjectData::Array(_, _) => format!("[{}", &self.class.descriptor),
            JObjectData::Fields(_) => self.class.descriptor.clone()
        };
    }

    /// Returns whether this object is assignable to the target descriptor, accounting for array objects.
    pub fn assignable_to(&self, desc: &str) -> bool{
        return JObject::assignable_to_rec(self.descriptor(), desc.to_owned());
    }

    fn assignable_to_rec(obj_desc: String, to: String) -> bool{
        return if obj_desc.starts_with("[") && to.starts_with("["){
            JObject::assignable_to_rec(obj_desc[1..].to_owned(), to[1..].to_owned())
        }else if !obj_desc.starts_with("[") && !to.starts_with("["){
            let from_class = heap::get_or_create_bt_class(obj_desc)
                .unwrap()
                .ensure_loaded()
                .unwrap();
            from_class.assignable_to(&to)
        }else{
            false
        }
    }
}

#[derive(Debug)]
pub enum JObjectData{
    Fields(HashMap<String, JValue>),
    Array(usize, Vec<JValue>)
}

impl JValue{
    pub fn assignable_to(&self, to: ClassRef) -> bool{
        return match self{
            JValue::Int(_) => vec!["Z", "B", "S", "C", "I"].contains(&to.descriptor.as_str()),
            JValue::Long(_) => to.descriptor == "J",
            JValue::Float(_) => to.descriptor == "F",
            JValue::Double(_) => to.descriptor == "D",
            JValue::Second => false,
            JValue::Reference(None) => to.descriptor.len() > 0, // any non-primitive
            JValue::Reference(Some(r)) => r.deref().class.assignable_to(&to.descriptor),
        };
    }

    pub fn class(&self) -> ClassRef{
        return match self{
            JValue::Int(_) => heap::bt_class_by_desc("I".to_owned()).unwrap(),
            JValue::Long(_) => heap::bt_class_by_desc("J".to_owned()).unwrap(),
            JValue::Float(_) => heap::bt_class_by_desc("F".to_owned()).unwrap(),
            JValue::Double(_) => heap::bt_class_by_desc("D".to_owned()).unwrap(),
            JValue::Second => { panic!("Tried to get the class of a long second value!") },
            JValue::Reference(None) => heap::bt_class_by_desc("Ljava/lang/Object;".to_owned()).unwrap(),
            JValue::Reference(Some(r)) => r.deref().class.clone(),
        };
    }

    pub fn default_value_for(desc: &str) -> JValue{
        if desc.starts_with("L") || desc.starts_with("["){
            return JValue::Reference(None);
        };
        return match desc{
            "Z" | "B" | "S" | "C" | "I" => JValue::Int(0),
            "J" => JValue::Long(0),
            "F" => JValue::Float(0.0),
            "D" => JValue::Double(0.0),
            _ => panic!("Tried to get default value of invalid descriptor {}", desc)
        };
    }
}