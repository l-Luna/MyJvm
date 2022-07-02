use runtime::native_impls::java_lang_class;
use runtime::{heap, objects};
use crate::runtime::{jvalue::JValue, interpreter::MethodResult};

pub fn builtin_unsafe_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "registerNatives()V" |
        "storeFence()V" => no_op_v,
        "arrayBaseOffset0(Ljava/lang/Class;)I" => const_0_i,
        "arrayIndexScale0(Ljava/lang/Class;)I" => const_1_i,
        "addressSize0()I" => address_size_i,
        "isBigEndian0()Z" |
        "unalignedAccess0()Z" => const_1_i,
        "objectFieldOffset1(Ljava/lang/Class;Ljava/lang/String;)J" => object_field_offset_by_name_j,
        _ => panic!("Unknown jdk.internal.misc.Unsafe native: {}", name_and_desc)
    };
}

fn no_op_v(_: Vec<JValue>) -> MethodResult{
    return MethodResult::Finish;
}

fn const_0_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(0));
}

fn const_0_j(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Long(0));
}

fn const_1_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(1));
}

fn address_size_i(_: Vec<JValue>) -> MethodResult{
    return MethodResult::FinishWithValue(JValue::Int(8));
}

// offsets are just field indexes in our impl
fn object_field_offset_by_name_j(params: Vec<JValue>) -> MethodResult{
    let class_desc = java_lang_class::get_desc_first(&params);
    let name = objects::java_string_to_rust_string(params[1]);
    let class = heap::get_or_create_bt_class(class_desc.unwrap())
        .unwrap()
        .ensure_loaded()
        .unwrap();

    // TODO: panic if not present?
    let mut i = 0;
    for field in &class.instance_fields{
        if field.name == name{
            break;
        }
        i += 1;
    }
    return MethodResult::FinishWithValue(JValue::Long(i));
}