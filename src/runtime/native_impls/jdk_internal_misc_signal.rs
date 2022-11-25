use runtime::interpreter::MethodResult;
use runtime::jvalue::JValue;
use runtime::objects;

use libc;

pub fn builtin_signal_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult {
    return match name_and_desc {
        "findSignal0(Ljava/lang/String;)I" => find_signal_i,
        "handle0(IJ)J" => handle_j,
        "raise0(I)V" => raise_v,
        _ => panic!("Unknown jdk.internal.misc.Signal native: {}", name_and_desc)
    }
}

fn find_signal_i(args: Vec<JValue>) -> MethodResult{
    let signame = objects::java_string_to_rust_string(args[0]);
    let code = match signame.as_str(){
        "INT" => libc::SIGINT,
        "TERM" => libc::SIGTERM,
        "SEGV" => libc::SIGSEGV,
        "ABRT" => libc::SIGABRT,
        _ => panic!("Unknown signal name: {}", signame)
    };
    return MethodResult::FinishWithValue(JValue::Int(code));
}

fn handle_j(args: Vec<JValue>) -> MethodResult{
    let JValue::Int(i) = args[0] else { return MethodResult::MachineError("Expected int for Signal.handle") };
    let JValue::Long(j) = args[1] else { return MethodResult::MachineError("Expected long for Signal.handle") };
    let r = unsafe { libc::signal(i, j as libc::sighandler_t) };
    return MethodResult::FinishWithValue(JValue::Long(r as i64));
}

fn raise_v(args: Vec<JValue>) -> MethodResult{
    let JValue::Int(i) = args[0] else { return MethodResult::MachineError("Expected int for Signal.raise") };
    unsafe { libc::raise(i); }
    return MethodResult::Finish;
}