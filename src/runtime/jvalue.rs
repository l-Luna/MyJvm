#[derive(Debug)]
pub enum JValue{
    Int(i32), // and other int-likes
    Long(i64),
    Float(f32),
    Double(f64),

    DoubleSecond,
    Void,

    Reference // TODO
}