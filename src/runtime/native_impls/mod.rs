use runtime::interpreter::MethodResult;
use runtime::jvalue::JValue;
use StackTrace;

mod java_lang_object;
mod java_lang_system;
mod java_lang_runtime;
mod java_lang_class;
mod java_lang_string;
mod java_lang_throwable;
mod java_lang_number;

mod java_io_file_descriptor;
mod java_io_file_io_stream;

mod jdk_internal_misc_unsafe;
mod jdk_internal_misc_cds;
mod jdk_internal_misc_vm;
mod jdk_internal_reflect_reflection;
mod jdk_internal_util_system_props;

pub fn builtin_native(owner: &String, name_and_desc: &String, trace: &StackTrace, args: Vec<JValue>) -> MethodResult{
    return match owner as &str{
        "java.lang.Object" => java_lang_object::builtin_object_native(name_and_desc)(args),
        "java.lang.System" => java_lang_system::builtin_system_native(name_and_desc)(args),
        "java.lang.Runtime" => java_lang_runtime::builtin_runtime_native(name_and_desc)(args),
        "java.lang.Class" => java_lang_class::builtin_class_native(name_and_desc)(args),
        "java.lang.StringUTF16" => java_lang_string::builtin_string_utf16_native(name_and_desc)(args),
        "java.lang.Throwable" => java_lang_throwable::builtin_throwable_native(name_and_desc)(args),
        "java.lang.Float" => java_lang_number::builtin_float_native(name_and_desc)(args),
        "java.lang.Double" => java_lang_number::builtin_double_native(name_and_desc)(args),

        "java.io.FileDescriptor" => java_io_file_descriptor::builtin_file_descriptor_native(name_and_desc)(args),
        "java.io.FileInputStream" => java_io_file_io_stream::builtin_file_input_stream_native(name_and_desc)(args),
        "java.io.FileOutputStream" => java_io_file_io_stream::builtin_file_output_stream_native(name_and_desc)(args),

        "jdk.internal.reflect.Reflection" => jdk_internal_reflect_reflection::run_reflection_native(name_and_desc, trace),

        "jdk.internal.misc.Unsafe" => jdk_internal_misc_unsafe::builtin_unsafe_native(name_and_desc)(args),
        "jdk.internal.misc.CDS" => jdk_internal_misc_cds::builtin_cds_native(name_and_desc)(args),
        "jdk.internal.misc.VM" => jdk_internal_misc_vm::builtin_vm_native(name_and_desc)(args),

        "jdk.internal.util.SystemProps$Raw" => jdk_internal_util_system_props::builtin_raw_system_props_native(name_and_desc)(args),

        _ => panic!("Unknown builtin native owner {} for method {}", owner, name_and_desc)
    }
}