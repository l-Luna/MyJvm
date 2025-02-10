use crate::runtime::{classes, heap, interpreter::MethodResult, jvalue::JValue, objects};

pub fn builtin_raw_system_props_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "platformProperties()[Ljava/lang/String;" => platform_properties,
        "vmProperties()[Ljava/lang/String;" => vm_properties,
        _ => panic!("Unknown jdk.internal.SystemProps.Raw native: {}", name_and_desc)
    };
}

fn platform_properties(_: Vec<JValue>) -> MethodResult{
    // check SystemProps.Raw for order
    let values = vec!["display_country".to_string(),
                      "display_language".to_string(),
                      "display_script".to_string(),
                      "display_variant".to_string(),
                      "file_encoding".to_string(),
                      "file_separator".to_string(),
                      "format_country".to_string(),
                      "format_language".to_string(),
                      "format_script".to_string(),
                      "format_variant".to_string(),
                      "ftp_nonProxyHosts".to_string(),
                      "ftp_proxyHost".to_string(),
                      "ftp_proxyPort".to_string(),
                      "http_nonProxyHosts".to_string(),
                      "http_proxyHost".to_string(),
                      "http_proxyPort".to_string(),
                      "https_proxyHost".to_string(),
                      "https_proxyPort".to_string(),
                      "java_io_tempdir".to_string(),
                      /*"line_separator"*/ "\n".to_string(),
                      "os_arch".to_string(),
                      "os_name".to_string(),
                      "os_version".to_string(),
                      "path_separator".to_string(),
                      "socks_nonProxyHosts".to_string(),
                      "socks_proxyHost".to_string(),
                      "socks_proxyPort".to_string(),
                      "sun_arch_abi".to_string(),
                      "sun_arch_data_model".to_string(),
                      "sun_cpu_endian".to_string(),
                      "sun_cpu_isalist".to_string(),
                      /*"sun_io_unicode_encoding"*/ "UTF-8".to_string(),
                      /*"sun_jnu_encoding"*/ "UTF-8".to_string(),
                      "sun_os_patch_level".to_string(),
                      /*"sun_sterr_encoding"*/ "UTF-8".to_string(),
                      /*"sun_stout_encoding"*/ "UTF-8".to_string(),
                      "user_dir".to_string(),
                      "user_home".to_string(),
                      "sun_os_patch_level".to_string()]
        .iter()
        .map(objects::synthesize_string)
        .map(heap::add_ref)
        .collect();
    return MethodResult::FinishWithValue(objects::create_new_array_of(objects::string_class(), values));
}

fn vm_properties(_: Vec<JValue>) -> MethodResult{
    // must set java.home
    let mut values: Vec<JValue> = vec!["java.home".to_string(),
                                       classes::find_java_home().unwrap(),
                                       "java.class.version".to_string(),
                                       "99.65535".to_string(),
                                       "sun.io.allowCriticalErrorMessageBox".to_string(),
                                       "true".to_string()]
        .iter()
        .map(objects::synthesize_string)
        .map(heap::add_ref)
        .collect();
    values.push(JValue::Reference(None));
    return MethodResult::FinishWithValue(objects::create_new_array_of(objects::string_class(), values));
}