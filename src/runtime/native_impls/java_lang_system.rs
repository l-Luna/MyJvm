use std::time::Instant;
use runtime::interpreter::MethodResult;
use runtime::jvalue::JValue;

static mut START: Option<Instant> = None;

pub fn builtin_system_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "registerNatives()V" => register_natives_v,
        "nanoTime()J" => nano_time_j,
        _ => panic!("Unknown java.lang.System native: {}", name_and_desc)
    };
}

fn register_natives_v(_: Vec<JValue>) -> MethodResult{
    unsafe{
        START = Some(Instant::now());
    }
    return MethodResult::Finish;
}

fn nano_time_j(_: Vec<JValue>) -> MethodResult{
    unsafe{
        return MethodResult::FinishWithValue(JValue::Long(Instant::now().duration_since(START.unwrap()).as_nanos() as i64));
    }
}