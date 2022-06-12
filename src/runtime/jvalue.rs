use super::{heap::JRef, classloader::ClassLoader};

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

#[derive(Debug)]
pub struct Class<'loader>{
    name: String,           // a.b.C
    descriptor: String,     // La/b/C; or I or [I...
    loader: &'loader dyn ClassLoader,
}