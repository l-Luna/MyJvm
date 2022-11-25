use std::sync::Arc;
use std::time::Instant;
use runtime::heap;
use runtime::interpreter::MethodResult;
use runtime::jvalue::{JObject, JObjectData, JValue};
use StackTrace;

static mut START: Option<Instant> = None;

pub fn builtin_system_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "registerNatives()V" => register_natives_v,
        "nanoTime()J" => nano_time_j,
        "arraycopy(Ljava/lang/Object;ILjava/lang/Object;II)V" => arraycopy_v,
        "setIn0(Ljava/io/InputStream;)V" => set_in_v,
        "setOut0(Ljava/io/PrintStream;)V" => set_out_v,
        "setErr0(Ljava/io/PrintStream;)V" => set_err_v,
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

fn arraycopy_v(args: Vec<JValue>) -> MethodResult{
    // TODO: proper exceptions
    let src_array = args[0];
    let src_idx = args[1];
    let dest_array = args[2];
    let dest_idx = args[3];
    let length = args[4];
    if let JValue::Reference(Some(src_ptr)) = src_array
    && let JValue::Reference(Some(dest_ptr)) = dest_array
    && let JValue::Int(src_idx) = src_idx
    && let JValue::Int(dest_idx) = dest_idx
    && let JValue::Int(length) = length
    && src_idx >= 0 && dest_idx >= 0 && length >= 0{
        let src_obj: Arc<JObject> = src_ptr.deref();
        let dest_obj = dest_ptr.deref();
        let src_idx = src_idx as usize; let desc_idx = dest_idx as usize; let length = length as usize;
        let mut write_lock = dest_obj.data.write().unwrap();
        if let JObjectData::Array(_, src_values) = &*src_obj.data.read().unwrap()
        && let JObjectData::Array(_, dest_values) = &mut *write_lock{
            for i in 0..length{
                dest_values[desc_idx + i] = src_values[src_idx + i].clone();
            }
            return MethodResult::Finish;
        };
    }
    println!("src is {:?}, dest is {:?}, src idx is {:?}, dest idx is {:?}, length is {:?}", src_array, src_idx, dest_array, dest_idx, length);
    return MethodResult::Throw(StackTrace::new(), "bad arraycopy args");
}

fn set_in_v(args: Vec<JValue>) -> MethodResult{
    let sys = heap::get_or_create_bt_class("Ljava/lang/String;".to_string()).unwrap().ensure_loaded().unwrap();
    for field in &sys.static_fields{
        let mut f = field.write().unwrap();
        if f.0.name == "in"{
            f.1 = args[0];
        }
    }
    return MethodResult::Finish;
}

fn set_out_v(args: Vec<JValue>) -> MethodResult{
    let sys = heap::get_or_create_bt_class("Ljava/lang/String;".to_string()).unwrap().ensure_loaded().unwrap();
    for field in &sys.static_fields{
        let mut f = field.write().unwrap();
        if f.0.name == "out"{
            f.1 = args[0];
        }
    }
    return MethodResult::Finish;
}

fn set_err_v(args: Vec<JValue>) -> MethodResult{
    let sys = heap::get_or_create_bt_class("Ljava/lang/String;".to_string()).unwrap().ensure_loaded().unwrap();
    for field in &sys.static_fields{
        let mut f = field.write().unwrap();
        if f.0.name == "err"{
            f.1 = args[0];
        }
    }
    return MethodResult::Finish;
}