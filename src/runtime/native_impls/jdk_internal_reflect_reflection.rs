use crate::runtime::interpreter::MethodResult;
use runtime::{heap, objects};
use StackTrace;

pub fn run_reflection_native(name_and_desc: &str, trace: &StackTrace) -> MethodResult{
    return match name_and_desc{
        "getCallerClass()Ljava/lang/Class;" => get_caller_class(trace),
        _ => panic!("Unknown jdk.internal.reflection.Reflect native: {}", name_and_desc)
    };
}

fn get_caller_class(trace: &StackTrace) -> MethodResult{
    let caller = &trace[1]; // no frame for getCallerClass created & skipping direct caller
    let caller_name = &caller.class_name;
    let as_descriptor = format!("L{};", caller_name.clone().replace(".", "/"));
    return MethodResult::FinishWithValue(heap::add_ref(objects::synthesize_class(&as_descriptor)));
}