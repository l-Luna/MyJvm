use std::sync::RwLock;
use std::thread;
use parser::classfile_structs::NameAndType;
use runtime::{heap, interpreter, objects};
use crate::runtime::{jvalue::JValue, interpreter::MethodResult};

// static DEFAULT_THREAD: OnceLock<JValue> = OnceLock::new();
static DEFAULT_THREAD: RwLock<Option<JValue>> = RwLock::new(None);

pub fn builtin_thread_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "registerNatives()V" => no_op_v,
        "currentThread()Ljava/lang/Thread;" => current_thread_thread,
        _ => panic!("Unknown java.lang.Thread native: {}", name_and_desc)
    };
}

fn no_op_v(_: Vec<JValue>) -> MethodResult{
    return MethodResult::Finish;
}

fn current_thread_thread(_: Vec<JValue>) -> MethodResult{
    // check if set...
    {
        let read = DEFAULT_THREAD.read().unwrap();
        if let Some(thread) = &* read{ // if so, just pass it along
            return MethodResult::FinishWithValue(thread.clone());
        }
    }
    // otherwise, create and set it
    let thread = synthesize_default_thread();
    {
        let mut write = DEFAULT_THREAD.write().unwrap();
        *write = Some(thread.clone());
    }
    // initialize it after, so the getCurrentThreadCall() in Thread::new isn't recursive
    initialize_default_thread(thread);
    return MethodResult::FinishWithValue(thread);
}

// Constructors for the default Thread and ThreadGroup

fn synthesize_default_thread_group() -> JValue{
    let class = objects::force_init_class("Ljava/lang/ThreadGroup;");
    let obj = objects::create_new(class.clone());
    let (init, owner) = class.special_method(&NameAndType{
        name: "<init>".to_string(),
        descriptor: "()V".to_string()
    }, "java/lang/ThreadGroup").unwrap();
    interpreter::execute(owner, init, vec![obj], interpreter::StackTrace::new());
    return obj;
}

fn synthesize_default_thread() -> JValue{
    let class = objects::force_init_class("Ljava/lang/Thread;");
    return objects::create_new(class.clone());
}

fn initialize_default_thread(obj: JValue){
    let group = synthesize_default_thread_group();

    let class = objects::force_init_class("Ljava/lang/Thread;");
    let name = heap::add_ref(objects::synthesize_string(&"main".to_string()));
    let (init, owner) = class.special_method(&NameAndType{
        name: "<init>".to_string(),
        descriptor: "(Ljava/lang/ThreadGroup;Ljava/lang/String;)V".to_string()
    }, "java/lang/Thread").unwrap();
    interpreter::execute(owner, init, vec![obj, group, name], interpreter::StackTrace::new());
}