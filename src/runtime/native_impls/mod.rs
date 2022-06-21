use runtime::interpreter::MethodResult;
use runtime::jvalue::JValue;

mod java_lang_system;

pub fn run_builtin_native(owner: &String, name_and_desc: &String, args: Vec<JValue>) -> MethodResult{
    return builtin_native(owner, name_and_desc)(args);
}

fn builtin_native(owner: &String, name_and_desc: &String) -> fn(Vec<JValue>) -> MethodResult{
    return match owner as &str{
        "java.lang.System" => java_lang_system::builtin_system_native(name_and_desc),
        _ => panic!("Unknown builtin native owner: {}", owner)
    }
}