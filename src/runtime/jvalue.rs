use std::ops::Deref;

use super::heap::JRef;

#[derive(Debug)]
pub enum JValue{
    Int(i32), // and other int-likes
    Long(i64),
    Float(f32),
    Double(f64),

    DoubleSecond,
    Void,

    Reference(JRef)
}

#[derive(Debug)]
pub struct JObject{
    
}