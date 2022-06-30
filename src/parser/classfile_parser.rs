use super::classfile_structs::*;
use crate::constants;

pub fn parse(file: &mut Vec<u8>) -> Result<Classfile, String>{
    if !expect_int(file, 0xCAFEBABE){
        return Err("Missing magic number!".to_owned())
    }

    let Some(minor_ver) = next_short(file) else { return Err("Missing minor version".to_owned()); };
    let Some(major_ver) = next_short(file) else { return Err("Missing major version".to_owned()); };

    let Some(raw_constants) = parse_constants(file) else { return Err("Unable to parse constant pool".to_owned()); };
    let constants = resolve_constants(raw_constants)?;

    let Some(flags) = next_short(file) else { return Err("Missing access flags".to_owned()); };
    check_class_flags(flags)?;

    let ConstantEntry::Class(this_class) = &constants[next_short_err(file)? as usize - 1]
        else { return Err("Unable to resolve this class's name".to_owned()); };
    let name: String = this_class.clone(); // own the string

    let super_idx = next_short_err(file)? as usize;
    let super_class: Option<String>;
    if super_idx == 0{
        super_class = None;
    }else{
        let ConstantEntry::Class(super_class_name) = &constants[super_idx - 1]
            else { return Err("Unable to resolve super class's name".to_owned()); };
        super_class = Some(super_class_name.clone());
    }

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

fn resolve_constants(raw_pool: Vec<RawConstantEntry>) -> Result<Vec<ConstantEntry>, String>{
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
                // TODO: split up into functions so we don't need this
                if let RawConstantEntry::Class(class_name_idx) = &raw_pool[*class_idx as usize - 1]
                && let RawConstantEntry::NameAndType(name_idx, descriptor_idx) = &raw_pool[*name_and_type_idx as usize - 1]
                && let RawConstantEntry::Utf8(class_name) = &raw_pool[*class_name_idx as usize - 1]
                && let RawConstantEntry::Utf8(name) = &raw_pool[*name_idx as usize - 1]
                && let RawConstantEntry::Utf8(descriptor) = &raw_pool[*descriptor_idx as usize - 1]{
                    ConstantEntry::MemberRef(MemberRef {
                        kind: tag_to_member_kind(tag)?,
                        owner_name: class_name.clone(),
                        name_and_type: NameAndType {
                            name: name.clone(),
                            descriptor: descriptor.clone(),
                        },
                    })
                }else{ return Err("Invalid MemberRef entry".to_owned()); }
            }

            RawConstantEntry::NameAndType(name_idx, descriptor_idx) => {
                if let RawConstantEntry::Utf8(name) = &raw_pool[*name_idx as usize - 1]
                && let RawConstantEntry::Utf8(descriptor) = &raw_pool[*descriptor_idx as usize - 1]{
                    ConstantEntry::NameAndType(NameAndType{
                        name: name.clone(),
                        descriptor: descriptor.clone(),
                    })
                }else{ return Err("Invalid NameAndType entry".to_owned()); }
            }

            RawConstantEntry::MethodHandle(dyn_ref_idx, member_ref_idx) => {
                // also same here
                if let RawConstantEntry::MemberRef(mtype, owner_class_idx, name_and_type_idx) = &raw_pool[*member_ref_idx as usize - 1]
                && let RawConstantEntry::Class(class_name_idx) = &raw_pool[*owner_class_idx as usize - 1]
                && let RawConstantEntry::Utf8(class_name) = &raw_pool[*class_name_idx as usize - 1]
                && let RawConstantEntry::NameAndType(name_idx, desc_idx) = &raw_pool[*name_and_type_idx as usize - 1]
                && let RawConstantEntry::Utf8(name) = &raw_pool[*name_idx as usize - 1]
                && let RawConstantEntry::Utf8(desc) = &raw_pool[*desc_idx as usize - 1]{
                    ConstantEntry::MethodHandle(
                        dyn_ref_index_to_type(dyn_ref_idx)?,
                        MemberRef{
                            kind: tag_to_member_kind(mtype)?,
                            owner_name: class_name.clone(),
                            name_and_type: NameAndType{
                                name: name.clone(),
                                descriptor: desc.clone(),
                            },
                        }
                    )
                }else{ return Err("Invalid MethodHandle entry".to_owned()); }
            },

            RawConstantEntry::Dynamic(_,_,_) => {
                // TODO: resolve against bootstrap table later
                ConstantEntry::Dynamic(Dynamic{
                    bootstrap: NameAndType{ name: "".to_string(), descriptor: "".to_string() },
                    value: NameAndType{ name: "".to_string(), descriptor: "".to_string() }
                })
            }

            _ => panic!("Bad conversion from {:?}", con)
        });
    }
    return Ok(ret);
}

fn tag_to_member_kind(tag: &u8) -> Result<MemberKind, String>{
    return match tag {
        9 => Ok(MemberKind::Field),
        10 => Ok(MemberKind::Method),
        11 => Ok(MemberKind::InterfaceMethod),
        _ => Err(format!("Invalid member reference tag {}", tag))
    }
}

fn dyn_ref_index_to_type(idx: &u8) -> Result<DynamicReferenceType, String>{
    return match idx {
        1 => Ok(DynamicReferenceType::GetField),
        2 => Ok(DynamicReferenceType::GetStatic),
        3 => Ok(DynamicReferenceType::PutField),
        4 => Ok(DynamicReferenceType::PutStatic),
        5 => Ok(DynamicReferenceType::InvokeVirtual),
        6 => Ok(DynamicReferenceType::InvokeStatic),
        7 => Ok(DynamicReferenceType::InvokeSpecial),
        8 => Ok(DynamicReferenceType::NewInvokeSpecial),
        9 => Ok(DynamicReferenceType::InvokeInterface),
        _ => Err(format!("Invalid dynamic reference type {}", idx))
    }
}

fn newarray_operand_to_descriptor(op: u8) -> String{
    return (match op{
        4 => "Z",
        5 => "C",
        6 => "F",
        7 => "D",
        8 => "B",
        9 => "S",
        10 => "I",
        11 => "J",
        _ => panic!("Unknown newarray array type {}!", op)
    }).to_owned();
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

        "LineNumberTable" => {
            let entries = next_short_err(&mut attr)?;
            let mut table = Vec::with_capacity(entries as usize);
            for _ in 0..entries{
                table.push(LineNumberMapping{
                    bytecode_idx: next_short_err(&mut attr)?,
                    line_number: next_short_err(&mut attr)?
                });
            }
            return Ok(Some(Attribute::LineNumberTable(table)));
        },

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
            //println!("Unknown attribute: {}", name);
            // spammy
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
        if let ConstantEntry::Class(exception_name) = &const_pool[exception_type_idx as usize - 1] {
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

// TODO: this is terrible
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
                    result.push((idx, Instruction::IConst(it as i32)));
                }else{
                    return Err("Missing byte operand of bipush".to_owned());
                }
            },
            constants::OP_SIPUSH => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IConst(it as i32)));
                }else{
                    return Err("Missing byte operand of sipush".to_owned());
                }
            },

            constants::OP_LCONST_0 => result.push((idx, Instruction::LConst(0))),
            constants::OP_LCONST_1 => result.push((idx, Instruction::LConst(1))),

            constants::OP_FCONST_0 => result.push((idx, Instruction::FConst(0.0))),
            constants::OP_FCONST_1 => result.push((idx, Instruction::FConst(1.0))),
            constants::OP_FCONST_2 => result.push((idx, Instruction::FConst(2.0))),

            constants::OP_DCONST_0 => result.push((idx, Instruction::DConst(0.0))),
            constants::OP_DCONST_1 => result.push((idx, Instruction::DConst(1.0))),

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

            constants::OP_FSTORE_0 => result.push((idx, Instruction::FStore(0))),
            constants::OP_FSTORE_1 => result.push((idx, Instruction::FStore(1))),
            constants::OP_FSTORE_2 => result.push((idx, Instruction::FStore(2))),
            constants::OP_FSTORE_3 => result.push((idx, Instruction::FStore(3))),
            constants::OP_FSTORE => {
                if let Some(it) = next_byte(bytecode){
                    result.push((idx, Instruction::FStore(it)));
                }else{
                    return Err("Missing byte operand of fstore".to_owned());
                }
            }

            constants::OP_DSTORE_0 => result.push((idx, Instruction::DStore(0))),
            constants::OP_DSTORE_1 => result.push((idx, Instruction::DStore(1))),
            constants::OP_DSTORE_2 => result.push((idx, Instruction::DStore(2))),
            constants::OP_DSTORE_3 => result.push((idx, Instruction::DStore(3))),
            constants::OP_DSTORE => {
                if let Some(it) = next_byte(bytecode){
                    result.push((idx, Instruction::DStore(it)));
                }else{
                    return Err("Missing byte operand of dstore".to_owned());
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

            constants::OP_IASTORE => result.push((idx, Instruction::IAStore)),
            constants::OP_LASTORE => result.push((idx, Instruction::LAStore)),
            constants::OP_FASTORE => result.push((idx, Instruction::FAStore)),
            constants::OP_DASTORE => result.push((idx, Instruction::DAStore)),
            constants::OP_AASTORE => result.push((idx, Instruction::AAStore)),
            constants::OP_BASTORE => result.push((idx, Instruction::BAStore)),
            constants::OP_CASTORE => result.push((idx, Instruction::CAStore)),
            constants::OP_SASTORE => result.push((idx, Instruction::SAStore)),

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

            constants::OP_FLOAD_0 => result.push((idx, Instruction::FLoad(0))),
            constants::OP_FLOAD_1 => result.push((idx, Instruction::FLoad(1))),
            constants::OP_FLOAD_2 => result.push((idx, Instruction::FLoad(2))),
            constants::OP_FLOAD_3 => result.push((idx, Instruction::FLoad(3))),
            constants::OP_FLOAD => {
                if let Some(it) = next_byte(bytecode){
                    result.push((idx, Instruction::FLoad(it)));
                }else{
                    return Err("Missing byte operand of fload".to_owned());
                }
            }

            constants::OP_DLOAD_0 => result.push((idx, Instruction::DLoad(0))),
            constants::OP_DLOAD_1 => result.push((idx, Instruction::DLoad(1))),
            constants::OP_DLOAD_2 => result.push((idx, Instruction::DLoad(2))),
            constants::OP_DLOAD_3 => result.push((idx, Instruction::DLoad(3))),
            constants::OP_DLOAD => {
                if let Some(it) = next_byte(bytecode){
                    result.push((idx, Instruction::DLoad(it)));
                }else{
                    return Err("Missing byte operand of dload".to_owned());
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

            constants::OP_IALOAD => result.push((idx, Instruction::IALoad)),
            constants::OP_LALOAD => result.push((idx, Instruction::LALoad)),
            constants::OP_FALOAD => result.push((idx, Instruction::FALoad)),
            constants::OP_DALOAD => result.push((idx, Instruction::DALoad)),
            constants::OP_AALOAD => result.push((idx, Instruction::AALoad)),
            constants::OP_BALOAD => result.push((idx, Instruction::BALoad)),
            constants::OP_CALOAD => result.push((idx, Instruction::CALoad)),
            constants::OP_SALOAD => result.push((idx, Instruction::SALoad)),

            constants::OP_POP => result.push((idx, Instruction::Pop)),
            constants::OP_POP2 => result.push((idx, Instruction::Pop2)),
            constants::OP_DUP => result.push((idx, Instruction::Dup)),
            constants::OP_DUP_X1 => result.push((idx, Instruction::DupX1)),
            constants::OP_DUP_X2 => result.push((idx, Instruction::DupX2)),
            constants::OP_DUP2 => result.push((idx, Instruction::Dup2)),
            constants::OP_DUP2_X1 => result.push((idx, Instruction::Dup2X1)),
            constants::OP_DUP2_X2 => result.push((idx, Instruction::Dup2X2)),
            constants::OP_SWAP => result.push((idx, Instruction::Swap)),

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
            constants::OP_FADD => result.push((idx, Instruction::FAdd)),
            constants::OP_DADD => result.push((idx, Instruction::DAdd)),

            constants::OP_ISUB => result.push((idx, Instruction::ISub)),
            constants::OP_LSUB => result.push((idx, Instruction::LSub)),
            constants::OP_FSUB => result.push((idx, Instruction::FSub)),
            constants::OP_DSUB => result.push((idx, Instruction::DSub)),

            constants::OP_IMUL => result.push((idx, Instruction::IMul)),
            constants::OP_LMUL => result.push((idx, Instruction::LMul)),
            constants::OP_FMUL => result.push((idx, Instruction::FMul)),
            constants::OP_DMUL => result.push((idx, Instruction::DMul)),

            constants::OP_IDIV => result.push((idx, Instruction::IDiv)),
            constants::OP_LDIV => result.push((idx, Instruction::LDiv)),
            constants::OP_FDIV => result.push((idx, Instruction::FDiv)),
            constants::OP_DDIV => result.push((idx, Instruction::DDiv)),

            constants::OP_IREM => result.push((idx, Instruction::IRem)),
            constants::OP_LREM => result.push((idx, Instruction::LRem)),
            constants::OP_FREM => result.push((idx, Instruction::FRem)),
            constants::OP_DREM => result.push((idx, Instruction::DRem)),

            constants::OP_INEG => result.push((idx, Instruction::INeg)),
            constants::OP_LNEG => result.push((idx, Instruction::LNeg)),
            constants::OP_FNEG => result.push((idx, Instruction::FNeg)),
            constants::OP_DNEG => result.push((idx, Instruction::DNeg)),

            constants::OP_ISHL => result.push((idx, Instruction::IShl)),
            constants::OP_LSHL => result.push((idx, Instruction::LShl)),
            constants::OP_ISHR => result.push((idx, Instruction::IShr)),
            constants::OP_LSHR => result.push((idx, Instruction::LShr)),
            constants::OP_IUSHR => result.push((idx, Instruction::IUshr)),
            constants::OP_LUSHR => result.push((idx, Instruction::LUshr)),

            constants::OP_IAND => result.push((idx, Instruction::IAnd)),
            constants::OP_LAND => result.push((idx, Instruction::LAnd)),
            constants::OP_IOR => result.push((idx, Instruction::IOr)),
            constants::OP_LOR => result.push((idx, Instruction::LOr)),
            constants::OP_IXOR => result.push((idx, Instruction::IXor)),
            constants::OP_LXOR => result.push((idx, Instruction::LXor)),

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
            },

            constants::OP_TABLE_SWITCH => {
                let pad = (4 - ((idx + 1) % 4)) % 4; // amazing
                for _ in 0..pad{
                    bytecode.remove(0);
                }
                if let Some(default_idx) = next_int(bytecode)
                && let Some(lo) = next_int(bytecode)
                && let Some(hi) = next_int(bytecode){
                    let n_jumps = (hi - lo + 1) as usize;
                    let mut jumps: Vec<i32> = Vec::with_capacity(n_jumps);
                    for _ in 0..n_jumps{
                        if let Some(off) = next_int(bytecode){
                            jumps.push(off);
                        }else{ return Err("Missing jump target of tableswitch".to_owned()); }
                    }
                    result.push((idx, Instruction::TableSwitch(default_idx, lo, hi, jumps)));
                }else{ return Err("Missing initial int operands of tableswitch".to_owned()); }
            },
            constants::OP_LOOKUP_SWITCH => {
                let pad = (4 - ((idx + 1) % 4)) % 4; // amazing
                for _ in 0..pad{
                    bytecode.remove(0);
                }
                if let Some(default_idx) = next_int(bytecode)
                && let Some(n_pairs) = next_int(bytecode){
                    let mut pairs: Vec<(i32, i32)> = Vec::with_capacity(n_pairs as usize);
                    for _ in 0..n_pairs{
                        if let Some(m) = next_int(bytecode)
                        && let Some(off) = next_int(bytecode){
                            pairs.push((m, off));
                        }else{ return Err("Missing match-offset pair of lookupswitch".to_owned()); }
                    }
                    result.push((idx, Instruction::LookupSwitch(default_idx, pairs)));
                }else{ return Err("Missing initial int operands of lookupswitch".to_owned()); }
            },

            constants::OP_LCMP => result.push((idx, Instruction::LCmp)),
            constants::OP_FCMPL => result.push((idx, Instruction::FCmpL)),
            constants::OP_FCMPG => result.push((idx, Instruction::FCmpG)),
            constants::OP_DCMPL => result.push((idx, Instruction::DCmpL)),
            constants::OP_DCMPG => result.push((idx, Instruction::DCmpG)),

            constants::OP_IF_EQ => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfEq(it as i32)));
                }else{ return Err("Missing short operand of ifeq".to_owned()); }
            },
            constants::OP_IF_NE => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfNe(it as i32)));
                }else{ return Err("Missing short operand of ifne".to_owned()); }
            },
            constants::OP_IF_LT => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfLt(it as i32)));
                }else{ return Err("Missing short operand of iflt".to_owned()); }
            },
            constants::OP_IF_GE => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfGe(it as i32)));
                }else{ return Err("Missing short operand of ifge".to_owned()); }
            },
            constants::OP_IF_GT => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfGt(it as i32)));
                }else{ return Err("Missing short operand of ifgt".to_owned()); }
            },
            constants::OP_IF_LE => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfLe(it as i32)));
                }else{ return Err("Missing short operand of ifle".to_owned()); }
            },

            constants::OP_IF_ICMP_EQ => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfICmpEq(it as i32)));
                }else{ return Err("Missing short operand of ificmpeq".to_owned()); }
            },
            constants::OP_IF_ICMP_NE => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfICmpNe(it as i32)));
                }else{ return Err("Missing short operand of ificmpne".to_owned()); }
            },
            constants::OP_IF_ICMP_LT => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfICmpLt(it as i32)));
                }else{ return Err("Missing short operand of ificmplt".to_owned()); }
            },
            constants::OP_IF_ICMP_GE => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfICmpGe(it as i32)));
                }else{ return Err("Missing short operand of ificmpge".to_owned()); }
            },
            constants::OP_IF_ICMP_GT => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfICmpGt(it as i32)));
                }else{ return Err("Missing short operand of ificmpgt".to_owned()); }
            },
            constants::OP_IF_ICMP_LE => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfICmpLe(it as i32)));
                }else{ return Err("Missing short operand of ificmple".to_owned()); }
            },

            constants::OP_IF_ACMP_EQ => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfACmpEq(it as i32)));
                }else{ return Err("Missing short operand of ifacmpeq".to_owned()); }
            },
            constants::OP_IF_ACMP_NE => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfACmpNe(it as i32)));
                }else{ return Err("Missing short operand of ifacmpne".to_owned()); }
            },
            constants::OP_IF_NULL => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfNull(it as i32)));
                }else{ return Err("Missing short operand of ifnull".to_owned()); }
            },
            constants::OP_IF_NONNULL => {
                if let Some(it) = next_sshort(bytecode){
                    result.push((idx, Instruction::IfNonnull(it as i32)));
                }else{ return Err("Missing short operand of ifnonnull".to_owned()); }
            },

            constants::OP_I2L => result.push((idx, Instruction::I2L)),
            constants::OP_I2F => result.push((idx, Instruction::I2F)),
            constants::OP_I2D => result.push((idx, Instruction::I2D)),

            constants::OP_L2I => result.push((idx, Instruction::L2I)),
            constants::OP_L2F => result.push((idx, Instruction::L2F)),
            constants::OP_L2D => result.push((idx, Instruction::L2D)),

            constants::OP_F2I => result.push((idx, Instruction::F2I)),
            constants::OP_F2L => result.push((idx, Instruction::F2L)),
            constants::OP_F2D => result.push((idx, Instruction::F2D)),

            constants::OP_D2I => result.push((idx, Instruction::D2I)),
            constants::OP_D2L => result.push((idx, Instruction::D2L)),
            constants::OP_D2F => result.push((idx, Instruction::D2F)),

            constants::OP_I2B => result.push((idx, Instruction::I2B)),
            constants::OP_I2C => result.push((idx, Instruction::I2C)),
            constants::OP_I2S => result.push((idx, Instruction::I2S)),

            constants::OP_IRETURN => result.push((idx, Instruction::IReturn)),
            constants::OP_LRETURN => result.push((idx, Instruction::LReturn)),
            constants::OP_FRETURN => result.push((idx, Instruction::FReturn)),
            constants::OP_DRETURN => result.push((idx, Instruction::DReturn)),
            constants::OP_ARETURN => result.push((idx, Instruction::AReturn)),
            constants::OP_RETURN => result.push((idx, Instruction::Return)),
            constants::OP_ATHROW => result.push((idx, Instruction::AThrow)),

            // TODO: better validation, split instructions?
            constants::OP_GET_STATIC |
            constants::OP_GET_FIELD => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::MemberRef(m) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::GetField(m.clone())));
                }else{ return Err("Missing short operand of getstatic/getfield or invalid const pool index".to_owned()); }
            }
            constants::OP_PUT_STATIC |
            constants::OP_PUT_FIELD => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::MemberRef(m) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::PutField(m.clone())));
                }else{ return Err("Missing short operand of putstatic/putfield or invalid const pool index".to_owned()); }
            }

            // TODO: cleanup (this whole thing :p)
            constants::OP_INVOKE_VIRTUAL => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::MemberRef(m) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::InvokeVirtual(m.clone())));
                }else{ return Err("Missing short operand of invokevirtual or invalid const pool index".to_owned()); }
            },
            constants::OP_INVOKE_SPECIAL => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::MemberRef(m) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::InvokeSpecial(m.clone())));
                }else{ return Err("Missing short operand of invokespecial or invalid const pool index".to_owned()); }
            },
            constants::OP_INVOKE_STATIC => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::MemberRef(m) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::InvokeStatic(m.clone())));
                }else{ return Err("Missing short operand of invokestatic or invalid const pool index".to_owned()); }
            },
            constants::OP_INVOKE_INTERFACE => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::MemberRef(m) = &const_pool[it as usize - 1]{
                    let _ = next_short(bytecode); // count, 0, both ignored
                    result.push((idx, Instruction::InvokeInterface(m.clone())));
                }else{ return Err("Missing short operand of invokeinterface or invalid const pool index".to_owned()); }
            },
            constants::OP_INVOKE_DYNAMIC => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::Dynamic(d) = &const_pool[it as usize - 1]{
                    expect_short(bytecode, 0);
                    result.push((idx, Instruction::InvokeDynamic(d.clone())));
                }else{ return Err("Missing short operand of invokedynamic or invalid const pool index".to_owned()); }
            },

            constants::OP_ARRAY_LENGTH => result.push((idx, Instruction::ArrayLength)),

            constants::OP_NEW => {
                if let Some(it) = next_short(bytecode)
                    && let ConstantEntry::Class(name) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::New(name.clone())));
                }else{ return Err("Missing short operand of new or invalid const pool index".to_owned()); }
            },
            constants::OP_NEWARRAY => {
                if let Some(it) = next_byte(bytecode){
                    result.push((idx, Instruction::NewArray(newarray_operand_to_descriptor(it))));
                }else{ return Err("Missing byte operand of newarray".to_owned()); }
            },
            constants::OP_ANEWARRAY => {
                if let Some(it) = next_short(bytecode)
                    && let ConstantEntry::Class(name) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::NewArray(format!("L{};", name.clone()))));
                }else{ return Err("Missing short operand of anewarray or invalid const pool index".to_owned()); }
            },

            constants::OP_CHECK_CAST => {
                if let Some(it) = next_short(bytecode)
                && let ConstantEntry::Class(name) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::CheckCast(name.clone())));
                }else{ return Err("Missing short operand of checkcast or invalid const pool index".to_owned()); }
            },
            constants::OP_INSTANCE_OF => {
                if let Some(it) = next_short(bytecode)
                    && let ConstantEntry::Class(name) = &const_pool[it as usize - 1]{
                    result.push((idx, Instruction::InstanceOf(name.clone())));
                }else{ return Err("Missing short operand of instanceof or invalid const pool index".to_owned()); }
            },

            constants::OP_MONITOR_ENTER => result.push((idx, Instruction::MonitorEnter)),
            constants::OP_MONITOR_EXIT => result.push((idx, Instruction::MonitorExit)),

            constants::OP_WIDE | constants::OP_MULTI_ANEWARRAY => result.push((idx, Instruction::TODO)),

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