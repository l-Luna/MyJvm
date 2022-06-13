use std::{collections::HashMap, ops::RangeBounds};

use super::{heap::JRef, class::{ClassRef, Class}};

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
    class: ClassRef,
    fields: HashMap<String, JValue>
}

impl JValue{
    fn assignable_to(&self, to: ClassRef) -> bool{
        return match self{
            JValue::Int(_) => vec!["Z", "B", "S", "C", "I"].contains(&to.descriptor.as_str()),
            JValue::Long(_) => to.descriptor == "J",
            JValue::Float(_) => to.descriptor == "F",
            JValue::Double(_) => to.descriptor == "D",
            JValue::Second => false,
            JValue::Reference(_) => todo!(),
        }
    }
}