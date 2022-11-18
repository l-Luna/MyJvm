use runtime::interpreter::MethodResult;
use runtime::jvalue::JValue;

pub fn builtin_file_input_stream_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "initIDs()V" => no_op_v,
        _ => panic!("Unknown java.io.FileInputStream native: {}", name_and_desc)
    };
}

pub fn builtin_file_output_stream_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "initIDs()V" => no_op_v,
        _ => panic!("Unknown java.io.FileOutputStream native: {}", name_and_desc)
    };
}

fn no_op_v(_: Vec<JValue>) -> MethodResult{
    return MethodResult::Finish;
}