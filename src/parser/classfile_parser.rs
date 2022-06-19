use super::classfile_structs::*;
use crate::constants;

pub fn parse(file: &mut Vec<u8>) -> Result<Classfile, String>{
    if !expect_int(file, 0xCAFEBABE){
        return Err("Missing magic number!".to_owned())
    }

    let Some(minor_ver) = next_short(file) else { return Err("Missing minor version".to_owned()); };
    let Some(major_ver) = next_short(file) else { return Err("Missing major version".to_owned()); };

    let Some(raw_constants) = parse_constants(file) else { return Err("Unable to parse constant pool".to_owned()); };
    let Some(constants) = resolve_constants(raw_constants) else { return Err("Unable to resolve constant pool".to_owned()); };

    let Some(flags) = next_short(file) else { return Err("Missing access flags".to_owned()); };
    check_class_flags(flags)?;

    let ConstantEntry::Class(this_class) = &constants[next_short_err(file)? as usize - 1]
        else { return Err("Unable to resolve this class's name".to_owned()); };
    let name: String = this_class.clone(); // own the string

    let ConstantEntry::Class(super_class) = &constants[next_short_err(file)? as usize - 1]
        else { return Err("Unable to resolve super class's name".to_owned()); };
    let super_class: String = super_class.clone(); // own the string

    let Some(ifaces_count) = next_short(file) else { return Err("Missing interfaces count".to_owned()); };
    let mut interfaces: Vec<String> = Vec::with_capacity(ifaces_count as usize);
    for _ in 0..ifaces_count{
        let ConstantEntry::Class(interface) = &constants[next_short_err(file)? as usize - 1]
            else {
                println!("{:?}", constants[next_short_err(file)? as usize - 1]);
                return Err("Unable to resolve interface name".to_owned());
            };
        interfaces.push(interface.clone());
    }

    let Some(field_count) = next_short(file) else { return Err("Missing field count".to_owned()); };
    let mut fields: Vec<FieldInfo> = Vec::with_capacity(field_count as usize);
    for _ in 0..field_count{
        fields.push(parse_member(file, &constants,
            |flags, name, desc, attributes| Ok(FieldInfo { flags, name, desc, attributes }))?);
    }

    let Some(method_count) = next_short(file) else { return Err("Missing method count".to_owned()); };
    let mut methods: Vec<MethodInfo> = Vec::with_capacity(method_count as usize);
    for _ in 0..method_count{
        methods.push(parse_member(file, &constants,
            |flags, name, desc, attributes| Ok(MethodInfo { flags, name, desc: parse_method_descriptor(desc)?, attributes }))?);
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

fn parse_attributes(file: &mut Vec<u8>, const_pool: &Vec<ConstantEntry>) -> Result<Vec<Attribute>, String>{
    let Some(count) = next_short(file) else { return Err("Missing attribute count".to_owned()); };
    let mut ret: Vec<Attribute> = Vec::with_capacity(count as usize);
    for _ in 0..count{
        let Some(name_idx) = next_short(file) else { return Err("Missing attribute name".to_owned()); };
        if let ConstantEntry::Utf8(name) = &const_pool[name_idx as usize - 1]{
            let Some(size) = next_uint(file) else { return Err("Missing attribute size".to_owned()); };
            let attr_data = next_vec(file, size as usize);
            if let Some(attr) = parse_attribute(attr_data, &const_pool, name)?{
                ret.push(attr);
            }
        }else{
            return Err("Attribute name index is invalid".to_owned());
        }
    }
    return Ok(ret);
}

fn parse_attribute(mut attr: Vec<u8>, const_pool: &Vec<ConstantEntry>, name: &String) -> Result<Option<Attribute>, String>{
    let name: &str = name;
    match name{
        "SourceFile" => {
            let ConstantEntry::Utf8(source) = &const_pool[next_short_err(&mut attr)? as usize - 1] else { return Err("Invalid SourceFile name index".to_owned()) };
            return Ok(Some(Attribute::SourceFile(source.clone())));
        }
        
        "Synthetic" => return Ok(Some(Attribute::Synthetic)),
        "Deprecated" => return Ok(Some(Attribute::Deprecated)),

        "Code" => {
            let max_stack = next_short_err(&mut attr)?;
            let max_locals = next_short_err(&mut attr)?;

            let bytecode_length = next_uint_err(&mut attr)?;
            let mut bytecode = next_vec(&mut attr, bytecode_length as usize);
            let bytecode = parse_bytecode(&mut bytecode, const_pool)?;

            let exception_handlers_count = next_short_err(&mut attr)?;
            let mut exception_handlers: Vec<ExceptionHandler> = Vec::with_capacity(exception_handlers_count as usize);
            for _ in 0..exception_handlers_count{
                exception_handlers.push(parse_exception_handler(&mut attr, const_pool)?);
            }

            let attributes = parse_attributes(&mut attr, const_pool)?;

            return Ok(Some(Attribute::Code(Code{
                max_stack,
                max_locals,
                bytecode,
                exception_handlers,
                attributes
            })));
        }

        _ => {
            println!("Unknown attribute: {}", name);
        }
    }
    return Ok(None); // unknown attributes are valid
} 

fn parse_exception_handler(attr: &mut Vec<u8>, const_pool: &Vec<ConstantEntry>) -> Result<ExceptionHandler, String>{
    let start_idx = next_short_err(attr)?;
    let end_idx = next_short_err(attr)?;
    let handler_idx = next_short_err(attr)?;
    let exception_type_idx = next_short_err(attr)?;
    return if exception_type_idx == 0 {
        Ok(ExceptionHandler {
            start_idx,
            end_idx,
            handler_idx,
            catch_type: None
        })
    } else {
        if let ConstantEntry::Utf8(exception_name) = &const_pool[exception_type_idx as usize - 1] {
            Ok(ExceptionHandler {
                start_idx,
                end_idx,
                handler_idx,
                catch_type: Some(exception_name.clone())
            })
        } else {
            Err("Exception handler type name index is invalid".to_owned())
        }
    }
}

fn parse_bytecode(bytecode: &mut Vec<u8>, const_pool: &Vec<ConstantEntry>) -> Result<Vec<(usize, Instruction)>, String>{
    let mut result: Vec<(usize, Instruction)> = Vec::new();
    let start_len = bytecode.len();
    while bytecode.len() > 0 {
        let idx = start_len - bytecode.len();
        let opcode = bytecode.remove(0);
        match opcode{
            constants::OP_NOP => { /* no-op */ },

            constants::OP_ACONST_NULL => result.push((idx, Instruction::AConstNull)),
            
            constants::OP_ICONST_M1 => result.push((idx, Instruction::IConst(-1))),
            constants::OP_ICONST_0 => result.push((idx, Instruction::IConst(0))),
            constants::OP_ICONST_1 => result.push((idx, Instruction::IConst(1))),
            constants::OP_ICONST_2 => result.push((idx, Instruction::IConst(2))),
            constants::OP_ICONST_3 => result.push((idx, Instruction::IConst(3))),
            constants::OP_ICONST_4 => result.push((idx, Instruction::IConst(4))),
            constants::OP_ICONST_5 => result.push((idx, Instruction::IConst(5))),
            constants::OP_BIPUSH => {
                if let Some(it) = next_sbyte(bytecode){
                    result.push((idx, Instruction::IConst(it)));
                }else{
                    return Err("Missing byte operand of bipush".to_owned());
                }
            }

            constants::OP_LCONST_0 => result.push((idx, Instruction::LConst(0))),
            constants::OP_LCONST_1 => result.push((idx, Instruction::LConst(1))),

            // TODO: check constant types
            constants::OP_LDC => {
                if let Some(it) = next_byte(bytecode){
                    let c = &const_pool[it as usize - 1];
                    result.push((idx, Instruction::Ldc(c.clone())));
                }else{
                    return Err("Missing byte operand of ldc".to_owned());
                }
            }
            constants::OP_LDC_W => {
                if let Some(it) = next_short(bytecode){
                    let c = &const_pool[it as usize - 1];
                    result.push((idx, Instruction::Ldc(c.clone())));
                }else{
                    return Err("Missing short operand of ldc_w".to_owned());
                }
            }
            constants::OP_LDC2_W => {
                if let Some(it) = next_short(bytecode){
                    let c = &const_pool[it as usize - 1];
                    result.push((idx, Instruction::Ldc(c.clone())));
                }else{
                    return Err("Missing short operand of ldc_w".to_owned());
                }
            }

            constants::OP_ISTORE_0 => result.push((idx, Instruction::IStore(0))),
            constants::OP_ISTORE_1 => result.push((idx, Instruction::IStore(1))),
            constants::OP_ISTORE_2 => result.push((idx, Instruction::IStore(2))),
            constants::OP_ISTORE_3 => result.push((idx, Instruction::IStore(3))),
            constants::OP_ISTORE => {
                if let Some(it) = next_byte(bytecode){
                    result.push((idx, Instruction::IStore(it)));
                }else{
                    return Err("Missing byte operand of istore".to_owned());
                }
            }

            constants::OP_LSTORE_0 => result.push((idx, Instruction::LStore(0))),
            constants::OP_LSTORE_1 => result.push((idx, Instruction::LStore(1))),
            constants::OP_LSTORE_2 => result.push((idx, Instruction::LStore(2))),
            constants::OP_LSTORE_3 => result.push((idx, Instruction::LStore(3))),
            constants::OP_LSTORE => {
                if let Some(it) = next_byte(bytecode){
                    result.push((idx, Instruction::LStore(it)));
                }else{
                    return Err("Missing byte operand of lstore".to_owned());
                }
            }

            constants::OP_ASTORE_0 => result.push((idx, Instruction::AStore(0))),
            constants::OP_ASTORE_1 => result.push((idx, Instruction::AStore(1))),
            constants::OP_ASTORE_2 => result.push((idx, Instruction::AStore(2))),
            constants::OP_ASTORE_3 => result.push((idx, Instruction::AStore(3))),
            constants::OP_ASTORE => {
                if let Some(it) = next_byte(bytecode){
                    result.push((idx, Instruction::AStore(it)));
                }else{
                    return Err("Missing byte operand of astore".to_owned());
                }
            }

            constants::OP_ILOAD_0 => result.push((idx, Instruction::ILoad(0))),
            constants::OP_ILOAD_1 => result.push((idx, Instruction::ILoad(1))),
            constants::OP_ILOAD_2 => result.push((idx, Instruction::ILoad(2))),
            constants::OP_ILOAD_3 => result.push((idx, Instruction::ILoad(3))),
            constants::OP_ILOAD => {
                if let Some(it) = next_byte(bytecode){
                    result.push((idx, Instruction::ILoad(it)));
                }else{
                    return Err("Missing byte operand of iload".to_owned());
                }
            }

            constants::OP_LLOAD_0 => result.push((idx, Instruction::LLoad(0))),
            constants::OP_LLOAD_1 => result.push((idx, Instruction::LLoad(1))),
            constants::OP_LLOAD_2 => result.push((idx, Instruction::LLoad(2))),
            constants::OP_LLOAD_3 => result.push((idx, Instruction::LLoad(3))),
            constants::OP_LLOAD => {
                if let Some(it) = next_byte(bytecode){
                    result.push((idx, Instruction::LLoad(it)));
                }else{
                    return Err("Missing byte operand of lload".to_owned());
                }
            }

            constants::OP_ALOAD_0 => result.push((idx, Instruction::ALoad(0))),
            constants::OP_ALOAD_1 => result.push((idx, Instruction::ALoad(1))),
            constants::OP_ALOAD_2 => result.push((idx, Instruction::ALoad(2))),
            constants::OP_ALOAD_3 => result.push((idx, Instruction::ALoad(3))),
            constants::OP_ALOAD => {
                if let Some(it) = next_byte(bytecode){
                    result.push((idx, Instruction::ALoad(it)));
                }else{
                    return Err("Missing byte operand of aload".to_owned());
                }
            }

            constants::OP_IINC => {
                if let Some(target) = next_byte(bytecode)
                && let Some(offset) = next_sbyte(bytecode){
                    result.push((idx, Instruction::IInc(target, offset)));
                }else{
                    return Err("Missing byte operand(s) of iinc".to_owned());
                }
            }

            constants::OP_IADD => result.push((idx, Instruction::IAdd)),
            constants::OP_LADD => result.push((idx, Instruction::LAdd)),
            constants::OP_ISUB => result.push((idx, Instruction::ISub)),
            constants::OP_LSUB => result.push((idx, Instruction::LSub)),

            constants::OP_GOTO => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::Goto(it as i32)));
                }else{
                    return Err("Missing short operand of goto".to_owned());
                }
            }
            constants::OP_GOTO_W => {
                if let Some(it) = next_int(bytecode){
                    result.push((idx, Instruction::Goto(it)));
                }else{
                    return Err("Missing uint operand of goto_w".to_owned());
                }
            }
            constants::OP_IF_EQ => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfEq(it as i32)));
                }else{
                    return Err("Missing short operand of ifeq".to_owned());
                }
            }
            constants::OP_IF_ICMP_GE => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfIcmpGe(it as i32)));
                }else{
                    return Err("Missing short operand of ifIicmpgt".to_owned());
                }
            }

            constants::OP_I2L => result.push((idx, Instruction::I2L)),
            constants::OP_L2I => result.push((idx, Instruction::L2I)),

            constants::OP_IRETURN => result.push((idx, Instruction::IReturn)),
            constants::OP_LRETURN => result.push((idx, Instruction::LReturn)),
            constants::OP_RETURN => result.push((idx, Instruction::Return)),

            // TODO: better validation, split instructions?
            constants::OP_GET_STATIC |
            constants::OP_GET_FIELD => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::MemberRef(m) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::GetField(m.clone())));
                }else{
                    return Err("Missing short operand of getstatic/getfield or invalid const pool index".to_owned());
                }
            }
            constants::OP_PUT_STATIC |
            constants::OP_PUT_FIELD => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::MemberRef(m) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::PutField(m.clone())));
                }else{
                    return Err("Missing short operand of putstatic/putfield or invalid const pool index".to_owned());
                }
            }

            // TODO: cleanup (this whole thing :p)
            constants::OP_INVOKE_VIRTUAL => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::MemberRef(m) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::InvokeVirtual(m.clone())));
                }else{
                    return Err("Missing short operand of invokevirtual or invalid const pool index".to_owned());
                }
            }
            constants::OP_INVOKE_SPECIAL => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::MemberRef(m) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::InvokeVirtual(m.clone())));
                }else{
                    return Err("Missing short operand of invokespecial or invalid const pool index".to_owned());
                }
            }
            constants::OP_INVOKE_STATIC => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::MemberRef(m) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::InvokeVirtual(m.clone())));
                }else{
                    return Err("Missing short operand of invokestatic or invalid const pool index".to_owned());
                }
            }
            constants::OP_INVOKE_INTERFACE => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::MemberRef(m) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::InvokeVirtual(m.clone())));
                }else{
                    return Err("Missing short operand of invokeinterface or invalid const pool index".to_owned());
                }
            }

            other => {
                //return Err("");
                // TODO: error, once all valid opcodes are handled
                println!("what's a {}?", other);
            }
        }
    }
    return Ok(result);
}

fn parse_member<T>(file: &mut Vec<u8>, const_pool: &Vec<ConstantEntry>, constr: fn(u16, String, String, Vec<Attribute>) -> Result<T, String>) -> Result<T, String>{
    let flags = next_short_err(file)?;
    
    let name_idx = next_short_err(file)?;
    let ConstantEntry::Utf8(name) = &const_pool[name_idx as usize - 1] else { return Err("Invalid field name index".to_owned()); };
    let name = name.clone();
    
    let desc_idx = next_short_err(file)?;
    let ConstantEntry::Utf8(desc) = &const_pool[desc_idx as usize - 1] else { return Err("Invalid field descriptor index".to_owned()); };
    let desc = desc.clone();
    
    let attrs = parse_attributes(file, &const_pool)?;

    return Ok(constr(flags, name, desc, attrs)?);
}

fn parse_method_descriptor(mut desc: String) -> Result<Vec<String>, String>{
    desc = desc.replace("(", ""); desc = desc.replace(")", ""); // don't *actually* matter
    let mut buffer = Vec::new();
    while desc.len() > 0{
        buffer.push(next_descriptor(&mut desc)?);
    };
    return Ok(buffer);
}

fn next_descriptor(desc: &mut String) -> Result<String, String>{
    let ch = desc.remove(0);
    match ch {
        'Z' | 'B' | 'S' | 'C' | 'I' | 'J' | 'F' | 'D' | 'V' => Ok(ch.to_string()),
        '[' => Ok("[".to_owned() + &next_descriptor(desc)?),
        'L' => {
            let mut next = String::with_capacity(3);
            next.push('L');
            while desc.len() > 0{
                let ch = desc.remove(0);
                next.push(ch);
                if ch == ';'{
                    break;
                }
            }
            Ok(next)
        },
        _ => Err(format!("Invalid descriptor item start: {}", ch))
    }
}

// validation methods

pub fn check_class_flags(flags: u16) -> Result<(), String>{
    if constants::bit_set(flags, constants::CLASS_ACC_INTERFACE){
        if !constants::bit_set(flags, constants::ACC_ABSTRACT){
            return Err("Interface class must be abstract".to_owned());
        }
        if constants::bit_set(flags, constants::ACC_FINAL){
            return Err("Interface class must not be final".to_owned());
        }
        if constants::bit_set(flags, constants::CLASS_ACC_SUPER){
            return Err("Interface class must not have \"super\" flag".to_owned());
        }
        if constants::bit_set(flags, constants::ACC_ENUM){
            return Err("Enum class must not be marked as interface".to_owned());
        }
        if constants::bit_set(flags, constants::CLASS_ACC_MODULE){
            return Err("Module info classfile must not be marked as interface".to_owned());
        }
    }else{
        if constants::bit_set(flags, constants::CLASS_ACC_ANNOTATION){
            return Err("Annotation class must be marked as interface".to_owned());
        }
    }
    if constants::bit_set(flags, constants::ACC_ABSTRACT) && constants::bit_set(flags, constants::ACC_FINAL){
        return Err("Class cannot be both abstract and final".to_owned());
    }
    // TODO: check modules have no other flags
    return Ok(());
}

// next data methods

fn next_byte(stream: &mut Vec<u8>) -> Option<u8>{
    if stream.len() == 0 {
        return None;
    }
    return Some(stream.remove(0));
}

fn next_sbyte(stream: &mut Vec<u8>) -> Option<i8>{
    if stream.len() == 0 {
        return None;
    }
    return Some(i8::from_be_bytes([stream.remove(0)]));
}

fn next_short(stream: &mut Vec<u8>) -> Option<u16>{
    return match (next_byte(stream), next_byte(stream)) {
        (Some(left), Some(right)) => Some(((left as u16) << 8) | (right as u16)),
        (_, _) => None
    };
}

fn next_sshort(stream: &mut Vec<u8>) -> Option<i16>{
    return match (next_byte(stream), next_byte(stream)) {
        (Some(left), Some(right)) => Some(i16::from_be_bytes([left, right])),
        (_, _) => None
    };
}

fn next_short_err(stream: &mut Vec<u8>) -> Result<u16, String>{
    return match next_short(stream) {
        Some(u) => Ok(u),
        None => Err("Unexpected end of file".to_owned())
    }
}

fn next_uint(stream: &mut Vec<u8>) -> Option<u32>{
    return match (next_short(stream), next_short(stream)) {
        (Some(left), Some(right)) => Some(((left as u32) << 16) | (right as u32)),
        (_, _) => None
    };
}

fn next_uint_err(stream: &mut Vec<u8>) -> Result<u32, String>{
    return match next_uint(stream) {
        Some(u) => Ok(u),
        None => Err("Unexpected end of file".to_owned())
    }
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