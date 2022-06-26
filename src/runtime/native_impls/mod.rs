use runtime::interpreter::MethodResult;
use runtime::jvalue::JValue;

mod java_lang_object;
mod java_lang_system;
mod java_lang_runtime;
mod java_lang_class;
mod java_lang_string;
mod java_lang_throwable;
mod java_lang_number;

mod jdk_internal_misc_unsafe;
mod jdk_internal_misc_cds;

pub fn run_builtin_native(owner: &String, name_and_desc: &String, args: Vec<JValue>) -> MethodResult{
    return builtin_native(owner, name_and_desc)(args);
}

fn builtin_native(owner: &String, name_and_desc: &String) -> fn(Vec<JValue>) -> MethodResult{
    return match owner as &str{
        "java.lang.Object" => java_lang_object::builtin_object_native(name_and_desc),
        "java.lang.System" => java_lang_system::builtin_system_native(name_and_desc),
        "java.lang.Runtime" => java_lang_runtime::builtin_runtime_native(name_and_desc),
        "java.lang.Class" => java_lang_class::builtin_class_native(name_and_desc),
        "java.lang.StringUTF16" => java_lang_string::builtin_string_utf16_native(name_and_desc),
        "java.lang.Throwable" => java_lang_throwable::builtin_throwable_native(name_and_desc),
        "java.lang.Float" => java_lang_number::builtin_float_native(name_and_desc),
        "java.lang.Double" => java_lang_number::builtin_double_native(name_and_desc),

        "jdk.internal.misc.Unsafe" => jdk_internal_misc_unsafe::builtin_unsafe_native(name_and_desc),
        "jdk.internal.misc.CDS" => jdk_internal_misc_cds::builtin_cds_native(name_and_desc),

        _ => panic!("Unknown builtin native owner {} for method {}", owner, name_and_desc)
    }
}