use crate::runtime::interpreter::MethodResult;
use crate::runtime::jvalue::JValue;

pub fn builtin_file_input_stream_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "initIDs()V" => no_op_v,
        _ => panic!("Unknown java.io.FileInputStream native: {}", name_and_desc)
    };
}

pub fn builtin_file_output_stream_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "initIDs()V" => no_op_v,
        "writeBytes([BIIZ)V" => write_bytes,
        _ => panic!("Unknown java.io.FileOutputStream native: {}", name_and_desc)
    };
}

fn no_op_v(_: Vec<JValue>) -> MethodResult{
    return MethodResult::Finish;
}

#[cfg(target_os = "windows")]
fn write_bytes(args: Vec<JValue>) -> MethodResult{
    let JValue::Reference(Some(this)) = args[0] else { panic!("Expected this for writeBytes") };
    let JValue::Reference(bytes) = args[1] else { panic!("Expected byte[] for writeBytes") };
    let JValue::Int(off) = args[2] else { panic!("Expected int for writeBytes") };
    let JValue::Int(len) = args[3] else { panic!("Expected int for writeBytes") };
    let JValue::Int(b_append) = args[3] else { panic!("Expected boolean for writeBytes") };

    // find the file descriptor...
    // get the file handle from that...
    // turn the byte array into a raw pointer + length...
    // syscall

    unsafe{
        //let mode = OsString::from("");
        //let file = libc::fdopen(fd, mode.raw);

    }

    return MethodResult::Finish;
}