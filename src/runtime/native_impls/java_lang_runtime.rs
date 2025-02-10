use crate::runtime::{heap, jvalue::JValue, interpreter::MethodResult};

pub fn builtin_runtime_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "availableProcessors()I" => available_processors_i,
        "gc()V" => gc_v,
        "maxMemory()J" => max_memory_j,
        _ => panic!("Unknown java.lang.Runtime native: {}", name_and_desc)
    };
}

fn available_processors_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(num_cpus::get() as i32));
}

fn gc_v(_: Vec<JValue>) -> MethodResult{
    heap::gc();
    return MethodResult::Finish;
}

fn max_memory_j(_: Vec<JValue>) -> MethodResult{
    // no inherent limit
    return MethodResult::FinishWithValue(JValue::Long(i64::MAX));
}