use runtime::native_impls::java_lang_class;
use runtime::{heap, objects};
use runtime::jvalue::JObjectData;
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
        "compareAndSetInt(Ljava/lang/Object;JII)Z" => compare_and_set_int_z,
        "compareAndSetLong(Ljava/lang/Object;JJJ)Z" => compare_and_set_long_z,
        "getReferenceVolatile(Ljava/lang/Object;J)Ljava/lang/Object;" => get_reference_volatile_obj,
        "compareAndSetReference(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z" => compare_and_set_reference_z,
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
    // Unsafe, Class<?>, String
    let class_desc = java_lang_class::get_class_desc(&params[1]);
    let name = objects::java_string_to_rust_string(params[2]);
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

// TODO: extract similarities
// leave until after JObject rework?
fn compare_and_set_int_z(params: Vec<JValue>) -> MethodResult{
    // Unsafe, Object to modify, long offset, int expected, int value to set
    let JValue::Long(idx) = params[2] else { return MethodResult::MachineError("expected long for compareAndSetInt") };
    let JValue::Int(expected) = params[3] else { return MethodResult::MachineError("expected int for compareAndSetInt") };
    let JValue::Int(to_set) = params[4] else { return MethodResult::MachineError("expected int for compareAndSetInt") };
    if let JValue::Reference(Some(r)) = params[1]{
        if let JObjectData::Fields(fields) = &mut *r.deref().data.write().unwrap(){
            let mut i = 0;
            let mut name: Option<String> = None;
            for field in &r.deref().class.instance_fields{
                if i == idx{
                    name = Some(field.name.clone());
                    break;
                }
                i += 1;
            }
            if let Some(f) = name{
                if let JValue::Int(v) = fields[&f]{
                    if v == expected{
                        fields.insert(f, JValue::Int(to_set));
                        return MethodResult::FinishWithValue(JValue::Int(1)); // true
                    }
                }
            }
        }
    }
    return MethodResult::FinishWithValue(JValue::Int(0)); // false
}

fn compare_and_set_reference_z(params: Vec<JValue>) -> MethodResult{
    // Unsafe, Object to modify, long offset, Object expected, Object value to set
    let JValue::Long(idx) = params[2] else { return MethodResult::MachineError("expected long for compareAndSetReference") };
    let JValue::Reference(expected) = params[3] else { return MethodResult::MachineError("expected int for compareAndSetReference") };
    let JValue::Reference(to_set) = params[4] else { return MethodResult::MachineError("expected int for compareAndSetReference") };
    if let JValue::Reference(Some(r)) = params[1]{
        match &mut *r.deref().data.write().unwrap(){
            JObjectData::Fields(fields) => {
                let mut i = 0;
                let mut name: Option<String> = None;
                for field in &r.deref().class.instance_fields{
                    if i == idx{
                        name = Some(field.name.clone());
                        break;
                    }
                    i += 1;
                }
                if let Some(f) = name{
                    if let JValue::Reference(v) = fields[&f]{
                        if v == expected{
                            fields.insert(f, JValue::Reference(to_set));
                            return MethodResult::FinishWithValue(JValue::Int(1)); // true
                        }
                    }
                }
            }
            JObjectData::Array(_, values) => {
                if let Some(JValue::Reference(v)) = values.get(idx as usize){
                    if *v == expected{
                        values[idx as usize] = JValue::Reference(to_set);
                        return MethodResult::FinishWithValue(JValue::Int(1)); // true
                    }
                }
            }
        }
    }
    return MethodResult::FinishWithValue(JValue::Int(0)); // false
}

fn compare_and_set_long_z(params: Vec<JValue>) -> MethodResult{
    // Unsafe, Object to modify, long offset, long expected, long value to set
    let JValue::Long(idx) = params[2] else { return MethodResult::MachineError("expected long for compareAndSetLong") };
    let JValue::Long(expected) = params[3] else { return MethodResult::MachineError("expected long for compareAndSetLong") };
    let JValue::Long(to_set) = params[4] else { return MethodResult::MachineError("expected long for compareAndSetLong") };
    if let JValue::Reference(Some(r)) = params[1]{
        if let JObjectData::Fields(fields) = &mut *r.deref().data.write().unwrap(){
            let mut i = 0;
            let mut name: Option<String> = None;
            for field in &r.deref().class.instance_fields{
                if i == idx{
                    name = Some(field.name.clone());
                    break;
                }
                i += 1;
            }
            if let Some(f) = name{
                if let JValue::Long(v) = fields[&f]{
                    if v == expected{
                        fields.insert(f, JValue::Long(to_set));
                        return MethodResult::FinishWithValue(JValue::Int(1)); // true
                    }
                }
            }
        }
    }
    return MethodResult::FinishWithValue(JValue::Int(0)); // false
}

fn get_reference_volatile_obj(params: Vec<JValue>) -> MethodResult{
    // Unsafe, Object to access, long offset
    let JValue::Long(idx) = params[2] else { return MethodResult::MachineError("expected long for getReferenceVolatile") };
    if let JValue::Reference(Some(r)) = params[1]{
        if let JObjectData::Fields(fields) = &mut *r.deref().data.write().unwrap(){
            let mut i = 0;
            let mut name: Option<String> = None;
            for field in &r.deref().class.instance_fields{
                if i == idx{
                    name = Some(field.name.clone());
                    break;
                }
                i += 1;
            }
            if let Some(f) = name{
                if let JValue::Reference(r) = fields[&f]{
                    return MethodResult::FinishWithValue(JValue::Reference(r));
                }
            }
        }
    }
    return MethodResult::FinishWithValue(JValue::Reference(None));
}