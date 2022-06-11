use super::heap::JRef;

#[derive(Debug, Clone, Copy)]
pub enum JValue{
    Int(i32), // and other int-likes
    Long(i64),
    Float(f32),
    Double(f64),

    Second,

    Reference(JRef)
}

#[derive(Debug)]
pub struct JObject{
    
}