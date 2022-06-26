use StackTrace;
use crate::runtime::{jvalue::JValue, interpreter::MethodResult};

// Float, Double

pub fn builtin_float_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "floatToRawIntBits(F)I" => float_to_raw_int_bits_i,
        "intBitsToFloat(I)F" => int_bits_to_float_i,
        _ => panic!("Unknown java.lang.Float native: {}", name_and_desc)
    };
}

pub fn builtin_double_native(name_and_desc: &str) -> fn(Vec<JValue>) -> MethodResult{
    return match name_and_desc{
        "doubleToRawLongBits(D)J" => double_to_raw_long_bits_l,
        "longBitsToDouble(J)D" => long_bits_to_double_d,
        _ => panic!("Unknown java.lang.Double native: {}", name_and_desc)
    };
}

fn float_to_raw_int_bits_i(args: Vec<JValue>) -> MethodResult{
    let float = args[0];
    return if let JValue::Float(f) = float{
        let bytes = f.to_be_bytes();
        let as_int = i32::from_be_bytes(bytes);
        MethodResult::FinishWithValue(JValue::Int(as_int))
    }else{
        MethodResult::Throw(StackTrace::new(), "floatToRawIntBits: not a float")
    }
}

fn int_bits_to_float_i(args: Vec<JValue>) -> MethodResult{
    let int = args[0];
    return if let JValue::Int(i) = int{
        let bytes = i.to_be_bytes();
        let as_float = f32::from_be_bytes(bytes);
        MethodResult::FinishWithValue(JValue::Float(as_float))
    }else{
        MethodResult::Throw(StackTrace::new(), "intBitsToFloat: not an int")
    }
}

fn double_to_raw_long_bits_l(args: Vec<JValue>) -> MethodResult{
    let double = args[0];
    return if let JValue::Double(d) = double{
        let bytes = d.to_be_bytes();
        let as_long = i64::from_be_bytes(bytes);
        MethodResult::FinishWithValue(JValue::Long(as_long))
    }else{
        MethodResult::Throw(StackTrace::new(), "doubleToRawLongBits: not a double")
    }
}

fn long_bits_to_double_d(args: Vec<JValue>) -> MethodResult{
    let long = args[0];
    return if let JValue::Long(l) = long{
        let bytes = l.to_be_bytes();
        let as_double = f64::from_be_bytes(bytes);
        MethodResult::FinishWithValue(JValue::Double(as_double))
    }else{
        MethodResult::Throw(StackTrace::new(), "longBitsToDouble: not a long")
    }
}