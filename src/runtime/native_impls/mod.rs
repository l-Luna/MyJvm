use runtime::interpreter::MethodResult;
use runtime::jvalue::JValue;

mod java_lang_object;
mod java_lang_system;
mod java_lang_runtime;
mod jdk_internal_misc_unsafe;

pub fn run_builtin_native(owner: &String, name_and_desc: &String, args: Vec<JValue>) -> MethodResult{
    return builtin_native(owner, name_and_desc)(args);
}

fn builtin_native(owner: &String, name_and_desc: &String) -> fn(Vec<JValue>) -> MethodResult{
    return match owner as &str{
        "java.lang.Object" => java_lang_object::builtin_object_native(name_and_desc),
        "java.lang.System" => java_lang_system::builtin_system_native(name_and_desc),
        "java.lang.Runtime" => java_lang_runtime::builtin_runtime_native(name_and_desc),
        "jdk.internal.misc.Unsafe" => jdk_internal_misc_unsafe::builtin_unsafe_native(name_and_desc),
        _ => panic!("Unknown builtin native owner {} for method {}", owner, name_and_desc)
    }
}