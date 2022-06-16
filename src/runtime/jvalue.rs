use std::collections::HashMap;

use super::{heap::JRef, class::ClassRef};

#[derive(Debug, Clone, Copy)]
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
    pub fields: HashMap<String, JValue>
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
            JValue::Reference(Some(r)) => r.deref().class.assignable_to(&to),
        }
    }

    pub fn default_value_for(desc: &str) -> JValue{
        if desc.starts_with("L"){
            return JValue::Reference(None);
        }
        return match desc{
            "Z" | "B" | "S" | "C" | "I" => JValue::Int(0),
            "J" => JValue::Long(0),
            "F" => JValue::Float(0.0),
            "D" => JValue::Double(0.0),
            _ => panic!("Tried to get default value of invalid descriptor")
        };
    }
}