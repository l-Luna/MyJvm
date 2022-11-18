use std::os::windows::io::AsRawHandle;
use runtime::interpreter::MethodResult;
use runtime::jvalue::JValue;

pub fn builtin_file_descriptor_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "initIDs()V" => no_op_v,
        "getHandle(I)J" => get_handle_j,
        "getAppend(I)Z" => get_append_z,
        _ => panic!("Unknown java.io.FileDescriptor native: {}", name_and_desc)
    };
}

fn no_op_v(_: Vec<JValue>) -> MethodResult{
    return MethodResult::Finish;
}

#[cfg(target_os = "windows")]
fn get_handle_j(args: Vec<JValue>) -> MethodResult{
    let JValue::Int(i) = args[0] else { return MethodResult::MachineError("bad args for FileDescriptor.getHandle") };
    return MethodResult::FinishWithValue(JValue::Long(match i{
        0 => std::io::stdin().as_raw_handle(),
        1 => std::io::stdout().as_raw_handle(),
        2 => std::io::stderr().as_raw_handle(),
        _ => panic!("Invalid file handle {}!", i)
    } as i64));
}

#[cfg(not(target_os = "windows"))]
fn get_handle_j(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Long(-1));
}

#[cfg(target_os = "windows")]
fn get_append_z(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(0));
}

#[cfg(not(target_os = "windows"))]
fn get_append_z(args: Vec<JValue>) -> MethodResult{
    todo!();
}