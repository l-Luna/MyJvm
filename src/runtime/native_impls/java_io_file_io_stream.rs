use std::ffi::c_void;
use std::ops::Deref;
use libc::{c_char, c_uint};
use crate::constants;
use crate::runtime::interpreter::MethodResult;
use crate::runtime::jvalue::{JObjectData, JValue};
use crate::runtime::objects;

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
    let JValue::Reference(Some(bytes)) = args[1] else { panic!("Expected byte[] for writeBytes") };
    let JValue::Int(off) = args[2] else { panic!("Expected int for writeBytes") };
    let JValue::Int(len) = args[3] else { panic!("Expected int for writeBytes") };
    let JValue::Int(b_append) = args[3] else { panic!("Expected boolean for writeBytes") };

    let obj = this.deref();
    let data = obj.data.read();
    let JObjectData::Fields(f) = &*data.unwrap() else { panic!("Expected this to have fields for writeBytes") };
    let Some(JValue::Reference(Some(fd_ref))) = f.get(constants::FOS_FD_FIELD_NAME) else { panic!("Expected this to have fd for writeBytes") };
    // ... but this is a reference to a FileDescriptor, with its own field `fd`...
    let fd_obj = fd_ref.deref();
    let fd_data = fd_obj.data.read();
    let JObjectData::Fields(fd_f) = &*fd_data.unwrap() else { panic!("Expected FieldDescriptor to have fields for writeBytes!") };
    let Some(JValue::Int(fd)) = fd_f.get(constants::FOS_FD_FIELD_NAME) else { panic!("Expected FD int for writeBytes!") };

    let bytes_obj = bytes.deref();
    let JObjectData::Array(_, arr) = &*bytes_obj.data.read().unwrap() else { panic!("a") };
    // ...but we want a direct array of bytes...
    let mut buf: Vec<c_char> = Vec::new();
    for value in arr {
        let JValue::Int(v) = &value else { panic!("aa") };
        buf.push(*v as c_char);
    }

    unsafe{
        //let mode = OsString::from("");
        //let file = libc::fdopen(fd, mode.raw);
        libc::write(*fd, (buf.as_ptr() as *const c_void).add(off as usize), len as c_uint);
    }

    return MethodResult::Finish;
}