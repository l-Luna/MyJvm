use classfile_structs::*;
use constants;

pub fn parse(file: &mut Vec<u8>) -> Result<Classfile, &str>{
    if !expect_int(file, 0xCAFEBABE){
        return Err("Missing magic number!")
    }

    let Some(minor_ver) = next_short(file) else { return Err("Missing minor version"); };
    let Some(major_ver) = next_short(file) else { return Err("Missing major version"); };

    let Some(raw_constants) = parse_constants(file) else { return Err("Unable to parse constant pool"); };
    let Some(constants) = resolve_constants(raw_constants) else { return Err("Unable to resolve constant pool"); };

    let Some(flags) = next_short(file) else { return Err("Missing access flags"); };
    constants::check_class_flags(flags)?;

    let ConstantEntry::Class(this_class) = &constants[next_short_err(file)? as usize - 1]
        else { return Err("Unable to resolve this class's name"); };
    let name: String = this_class.clone(); // own the string

    let ConstantEntry::Class(super_class) = &constants[next_short_err(file)? as usize - 1]
        else { return Err("Unable to resolve super class's name"); };
    let super_class: String = super_class.clone(); // own the string

    let Some(ifaces_count) = next_short(file) else { return Err("Missing interfaces count"); };
    let mut interfaces: Vec<String> = Vec::with_capacity(ifaces_count as usize);
    for _ in 0..ifaces_count {
        let ConstantEntry::Utf8(interface) = &constants[next_short_err(file)? as usize - 1]
            else { return Err("Unable to resolve interface name"); };
        interfaces.push(interface.clone());
    }

    let Some(field_count) = next_short(file) else { return Err("Missing field count"); };
    let mut fields: Vec<FieldInfo> = Vec::with_capacity(field_count as usize);
    for _ in 0..field_count {
        fields.push(parse_member(file, &constants,
            |flags, name, desc, attributes| FieldInfo { flags, name, desc, attributes })?);
    }

    let Some(method_count) = next_short(file) else { return Err("Missing method count"); };
    let mut methods: Vec<MethodInfo> = Vec::with_capacity(method_count as usize);
    for _ in 0..method_count {
        methods.push(parse_member(file, &constants,
            |flags, name, desc, attributes| MethodInfo { flags, name, desc, attributes })?);
    }

    let attributes = parse_attributes(file, &constants)?;

    // TODO: check EOF

    return Ok(Classfile{
        major_ver,
        minor_ver,
        constants,
        flags,
        name,
        super_class,
        interfaces,
        fields,
        methods,
        attributes
    });
}

fn parse_constants(file: &mut Vec<u8>) -> Option<Vec<RawConstantEntry>>{
    let mut pool: Vec<RawConstantEntry> = Vec::new();
    let count = next_short(file)?;
    let mut i = 0;
    while i < count - 1 {
        let tag = next_byte(file)?;
        match tag{
            1 => pool.push(RawConstantEntry::Utf8(parse_modified_utf8(file)?)),
            3 => pool.push(RawConstantEntry::Integer(next_int(file)?)),
            4 => pool.push(RawConstantEntry::Float(next_float(file)?)),
            5 => {
                i += 1;
                pool.push(RawConstantEntry::Long(next_long(file)?));
                pool.push(RawConstantEntry::LongSecond);
            },
            6 => {
                i += 1;
                pool.push(RawConstantEntry::Double(next_double(file)?));
                pool.push(RawConstantEntry::LongSecond);
            },
            7 => pool.push(RawConstantEntry::Class(next_short(file)?)),
            8 => pool.push(RawConstantEntry::StringConst(next_short(file)?)),
            9 | 10 | 11 => pool.push(RawConstantEntry::MemberRef(tag, next_short(file)?, next_short(file)?)),
            12 => pool.push(RawConstantEntry::NameAndType(next_short(file)?, next_short(file)?)),
            15 => pool.push(RawConstantEntry::MethodHandle(next_byte(file)?, next_short(file)?)),
            16 => pool.push(RawConstantEntry::MethodType(next_short(file)?)),
            17 | 18 => pool.push(RawConstantEntry::Dynamic(tag, next_short(file)?, next_short(file)?)),
            19 => pool.push(RawConstantEntry::Module(next_short(file)?)),
            20 => pool.push(RawConstantEntry::Package(next_short(file)?)),
            _ => {
                panic!("Invalid tag: {}", tag);
            }
        };
        i += 1;
    }
    return Some(pool);
}

pub fn parse_modified_utf8(file: &mut Vec<u8>) -> Option<String>{
    let len = next_short(file)?;
    let mut buffer = next_vec(file, len as usize);
    let mut current: String = String::with_capacity(len as usize);
    while !buffer.is_empty(){
        let next = next_byte(&mut buffer)?;
        // :(
        // TODO: parse 6-byte form
        // the spec is ambiguous on that?
        if next & 0b10000000 == 0{
            // first bit = 0 -> single byte
            current.push(char::from(next));
        }else if next & 0b111_00000 == 0b110_00000{
            // first three bits = 110 -> two bytes
            let b = next_byte(&mut buffer)?;
            current.push(char::from_u32(((next as u32 & 0x1f) << 6) + (b as u32 & 0x3f))?);
        }else if next & 0b1111_0000 == 0b1110_0000{
            // first four bits = 1110 -> three bytes
            let b = next_byte(&mut buffer)?;
            let c = next_byte(&mut buffer)?;
            current.push(char::from_u32(((next as u32 & 0xf) << 12) + ((b as u32 & 0x3f) << 6) + (c as u32 & 0x3f))?);
        }
    }
    return Some(current);
}

fn resolve_constants(raw_pool: Vec<RawConstantEntry>) -> Option<Vec<ConstantEntry>>{
    let mut ret: Vec<ConstantEntry> = Vec::with_capacity(raw_pool.len());
    for con in &raw_pool {
        ret.push(match con {
            RawConstantEntry::LongSecond => ConstantEntry::LongSecond,

            RawConstantEntry::Utf8(s) => ConstantEntry::Utf8(s.clone()),
            RawConstantEntry::Integer(i) => ConstantEntry::Integer(*i),
            RawConstantEntry::Float(f) => ConstantEntry::Float(*f),
            RawConstantEntry::Long(l) => ConstantEntry::Long(*l),
            RawConstantEntry::Double(d) => ConstantEntry::Double(*d),

            RawConstantEntry::Class(idx) if let RawConstantEntry::Utf8(s) = &raw_pool[*idx as usize - 1]
                => ConstantEntry::Class(s.clone()),
            RawConstantEntry::StringConst(idx) if let RawConstantEntry::Utf8(s) = &raw_pool[*idx as usize - 1]
                => ConstantEntry::StringConst(s.clone()),
            RawConstantEntry::MethodType(idx) if let RawConstantEntry::Utf8(s) = &raw_pool[*idx as usize - 1]
                => ConstantEntry::MethodType(s.clone()),
            RawConstantEntry::Module(idx) if let RawConstantEntry::Utf8(s) = &raw_pool[*idx as usize - 1]
                => ConstantEntry::Module(s.clone()),
            RawConstantEntry::Package(idx) if let RawConstantEntry::Utf8(s) = &raw_pool[*idx as usize - 1]
                => ConstantEntry::Package(s.clone()),

            RawConstantEntry::MemberRef(tag, class_idx, name_and_type_idx) => {
                // TODO: split up into functions so we don't need... this
                let mut ret: Option<ConstantEntry> = None;
                if let RawConstantEntry::Class(class_name_idx) = &raw_pool[*class_idx as usize - 1]{
                    if let RawConstantEntry::NameAndType(name_idx, descriptor_idx) = &raw_pool[*name_and_type_idx as usize - 1]{
                        if let RawConstantEntry::Utf8(class_name) = &raw_pool[*class_name_idx as usize - 1]{
                            if let RawConstantEntry::Utf8(name) = &raw_pool[*name_idx as usize - 1]{
                                if let RawConstantEntry::Utf8(descriptor) = &raw_pool[*descriptor_idx as usize - 1]{
                                    ret = Some(ConstantEntry::MemberRef(MemberRef {
                                        kind: tag_to_member_kind(tag)?,
                                        owner_name: class_name.clone(),
                                        name_and_type: NameAndType {
                                            name: name.clone(),
                                            descriptor: descriptor.clone(),
                                        },
                                    }));
                                }
                            }
                        }
                    }
                }
                ret?
            }

            RawConstantEntry::NameAndType(name_idx, descriptor_idx) => {
                let mut ret: Option<ConstantEntry> = None;
                if let RawConstantEntry::Utf8(name) = &raw_pool[*name_idx as usize - 1] {
                    if let RawConstantEntry::Utf8(descriptor) = &raw_pool[*descriptor_idx as usize - 1] {
                        ret = Some(ConstantEntry::NameAndType(NameAndType {
                            name: name.clone(),
                            descriptor: descriptor.clone(),
                        }));
                    }
                }
                ret?
            }

            RawConstantEntry::MethodHandle(dyn_ref_idx, member_ref_idx) => {
                // also... same here
                let mut ret: Option<ConstantEntry> = None;
                if let RawConstantEntry::MemberRef(mtype, owner_class_idx, name_and_type_idx) = &raw_pool[*member_ref_idx as usize - 1] {
                    if let RawConstantEntry::Class(class_name_idx) = &raw_pool[*owner_class_idx as usize - 1] {
                        if let RawConstantEntry::Utf8(class_name) = &raw_pool[*class_name_idx as usize - 1] {
                            if let RawConstantEntry::NameAndType(name_idx, desc_idx) = &raw_pool[*name_and_type_idx as usize - 1] {
                                if let RawConstantEntry::Utf8(name) = &raw_pool[*name_idx as usize - 1] {
                                    if let RawConstantEntry::Utf8(desc) = &raw_pool[*desc_idx as usize - 1] {
                                        ret = Some(ConstantEntry::MethodHandle(
                                            dyn_ref_index_to_type(dyn_ref_idx)?,
                                            MemberRef {
                                                kind: tag_to_member_kind(mtype)?,
                                                owner_name: class_name.clone(),
                                                name_and_type: NameAndType {
                                                    name: name.clone(),
                                                    descriptor: desc.clone()
                                                }
                                            }
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                ret?
            }

            // TODO: Dynamic

            _ => panic!("Bad conversion from {:?}", con)
        });
    }
    return Some(ret);
}

fn tag_to_member_kind(tag: &u8) -> Option<MemberKind>{
    return match tag {
        9 => Some(MemberKind::Field),
        10 => Some(MemberKind::Method),
        11 => Some(MemberKind::InterfaceMethod),
        _ => None
    }
}

fn dyn_ref_index_to_type(idx: &u8) -> Option<DynamicReferenceType>{
    return match idx {
        // TODO: is this correct?
        0 => Some(DynamicReferenceType::GetField),
        1 => Some(DynamicReferenceType::GetStatic),
        2 => Some(DynamicReferenceType::PutField),
        3 => Some(DynamicReferenceType::PutStatic),
        4 => Some(DynamicReferenceType::InvokeVirtual),
        5 => Some(DynamicReferenceType::NewInvokeSpecial),
        6 => Some(DynamicReferenceType::InvokeStatic),
        7 => Some(DynamicReferenceType::InvokeSpecial),
        _ => None
    }
}

fn parse_attributes(file: &mut Vec<u8>, const_pool: &Vec<ConstantEntry>) -> Result<Vec<Attribute>, &'static str>{
    let Some(count) = next_short(file) else { return Err("Missing attribute count"); };
    let mut ret: Vec<Attribute> = Vec::with_capacity(count as usize);
    for _ in 0..count{
        let Some(name_idx) = next_short(file) else { return Err("Missing attribute name"); };
        if let ConstantEntry::Utf8(name) = &const_pool[name_idx as usize]{
            let Some(size) = next_uint(file) else { return Err("Missing attribute size"); };
            let attr_data = next_vec(file, size as usize);
            if let Some(attr) = parse_attribute(attr_data, &const_pool, name)?{
                ret.push(attr);
            }
        }else{
            return Err("Attribute name index is invalid");
        }
    }
    return Ok(ret);
}

fn parse_attribute(mut attr: Vec<u8>, const_pool: &Vec<ConstantEntry>, name: &String) -> Result<Option<Attribute>, &'static str>{
    let name: &str = name;
    match name {
        "SourceFile" => {
            let ConstantEntry::Utf8(source) = &const_pool[next_short_err(&mut attr)? as usize - 1] else { return Err("Invalid SourceFile name index") };
            return Ok(Some(Attribute::SourceFile(source.clone())));
        }
        
        "Synthetic" => return Ok(Some(Attribute::Synthetic)),
        "Deprecated" => return Ok(Some(Attribute::Deprecated)),

        _ => {}
    }
    return Ok(None); // unknown attributes are valid
} 

fn parse_member<T>(file: &mut Vec<u8>, const_pool: &Vec<ConstantEntry>, constr: fn(u16, String, String, Vec<Attribute>) -> T) -> Result<T, &'static str>{
    let flags = next_short_err(file)?;
    
    let name_idx = next_short_err(file)?;
    let ConstantEntry::Utf8(name) = &const_pool[name_idx as usize - 1] else { return Err("Invalid field name index"); };
    let name = name.clone();
    
    let desc_idx = next_short_err(file)?;
    let ConstantEntry::Utf8(desc) = &const_pool[desc_idx as usize - 1] else { return Err("Invalid field descriptor index"); };
    let desc = desc.clone();
    
    let attrs = parse_attributes(file, &const_pool)?;

    return Ok(constr(flags, name, desc, attrs));
}

// next data methods

fn next_byte(stream: &mut Vec<u8>) -> Option<u8>{
    if stream.len() == 0 {
        return None;
    }
    return Some(stream.remove(0));
}

fn next_short(stream: &mut Vec<u8>) -> Option<u16>{
    return match (next_byte(stream), next_byte(stream)) {
        (Some(left), Some(right)) => Some(((left as u16) << 8) | (right as u16)),
        (_, _) => None
    };
}

fn next_short_err(stream: &mut Vec<u8>) -> Result<u16, &'static str>{
    return match next_short(stream) {
        Some(u) => Ok(u),
        None => Err("Unexpected end of file")
    }
}

fn next_uint(stream: &mut Vec<u8>) -> Option<u32>{
    return match (next_short(stream), next_short(stream)) {
        (Some(left), Some(right)) => Some(((left as u32) << 16) | (right as u32)),
        (_, _) => None
    };
}

fn next_int(stream: &mut Vec<u8>) -> Option<i32>{
    return match (next_byte(stream), next_byte(stream), next_byte(stream), next_byte(stream)) {
        (Some(a), Some(b), Some(c), Some(d)) => Some(i32::from_be_bytes([a, b, c, d])),
        (_, _, _, _) => None
    };
}

fn next_float(stream: &mut Vec<u8>) -> Option<f32>{
    return match (next_byte(stream), next_byte(stream), next_byte(stream), next_byte(stream)) {
        (Some(a), Some(b), Some(c), Some(d)) => Some(f32::from_be_bytes([a, b, c, d])),
        (_, _, _, _) => None
    };
}

fn next_long(stream: &mut Vec<u8>) -> Option<i64>{
    return match (next_byte(stream), next_byte(stream), next_byte(stream), next_byte(stream), next_byte(stream), next_byte(stream), next_byte(stream), next_byte(stream)) {
        (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h)) => Some(i64::from_be_bytes([a, b, c, d, e, f, g, h])),
        (_, _, _, _, _, _, _, _) => None
    };
    }

// TODO: adjust for NaNs
fn next_double(stream: &mut Vec<u8>) -> Option<f64>{
    return match (next_byte(stream), next_byte(stream), next_byte(stream), next_byte(stream), next_byte(stream), next_byte(stream), next_byte(stream), next_byte(stream)) {
        (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h)) => Some(f64::from_be_bytes([a, b, c, d, e, f, g, h])),
        (_, _, _, _, _, _, _, _) => None
    };
}

fn next_vec<T>(stream: &mut Vec<T>, amount: usize) -> Vec<T>{
    let mut ret: Vec<T> = Vec::with_capacity(amount);
    for _ in 0..amount{
        ret.push(stream.remove(0));
    }
    return ret;
}

// expect data methods

fn expect_byte(stream: &mut Vec<u8>, expected: u8) -> bool{
    expect_vec(stream, vec![expected])
}

fn expect_short(stream: &mut Vec<u8>, expected: u16) -> bool{
    expect_vec(stream, vec![(expected >> 8) as u8, expected as u8])
}

fn expect_int(stream: &mut Vec<u8>, expected: u32) -> bool{
    expect_vec(stream, vec![(expected >> 24) as u8, (expected >> 16) as u8, (expected >> 8) as u8, expected as u8])
}

fn expect_vec(stream: &mut Vec<u8>, expected: Vec<u8>) -> bool{
    for val in expected {
        match next_byte(stream) {
            None => return false,
            Some(b) => if b != val {
                return false;
            },
        }
    }
    return true;
}