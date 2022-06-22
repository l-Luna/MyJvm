use crate::runtime::{jvalue::JValue, interpreter::MethodResult};

pub fn builtin_runtime_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "availableProcessors()I" => available_processors_i,
        _ => panic!("Unknown java.lang.Runtime native: {}", name_and_desc)
    };
}

fn available_processors_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(num_cpus::get() as i32));
}