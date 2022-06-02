use classfile_structs::{Classfile, ConstantEntry, RawConstantEntry};

pub fn parse(file: &mut Vec<u8>) -> Result<Classfile, &str>{
    if !expect_int(file, 0xCAFEBABE){
        return Err("Missing magic number!")
    }

    let Some(minor_ver) = next_short(file) else { return Err("Missing minor version"); };
    let Some(major_ver) = next_short(file) else { return Err("Missing major version"); };

    let Some(constants) = parse_constants(file) else { return Err("Unable to parse constant pool"); };

    let Some(flags) = next_short(file) else { return Err("Missing access flags"); };

    return Ok(Classfile{
        major_ver,
        minor_ver,
        constants,
        flags,
        this_class: 0,
        super_class: 0,
        interfaces: vec![],
        fields: vec![],
        methods: vec![],
        attributes: vec![]
    });
}

fn parse_constants(file: &mut Vec<u8>) -> Option<Vec<RawConstantEntry>>{
    let mut pool: Vec<RawConstantEntry> = Vec::new();
    let count = next_short(file)?;
    dbg!(count);
    for i in 0..(count - 1) {
        dbg!(i);
        dbg!(&pool);
        let tag = next_byte(file)?;
        pool.push(match tag{
            1 => RawConstantEntry::Utf8(parse_modified_utf8(file)?),
            3 => RawConstantEntry::Integer(next_int(file)?),
            4 => RawConstantEntry::Float(next_float(file)?),
            5 => RawConstantEntry::Long(next_long(file)?),
            6 => RawConstantEntry::Double(next_double(file)?),
            7 => RawConstantEntry::Class(next_short(file)?),
            8 => RawConstantEntry::StringConst(next_short(file)?),
            9 | 10 | 11 => RawConstantEntry::MemberRef(tag, next_short(file)?, next_short(file)?),
            12 => RawConstantEntry::NameAndType(next_short(file)?, next_short(file)?),
            15 => RawConstantEntry::MethodHandle(next_byte(file)?, next_short(file)?),
            16 => RawConstantEntry::MethodType(next_short(file)?),
            17 | 18 => RawConstantEntry::Dynamic(tag, next_short(file)?, next_short(file)?),
            19 => RawConstantEntry::Module(next_short(file)?),
            20 => RawConstantEntry::Package(next_short(file)?),
            _ => {
                // uhhhhhhhhh
                //panic!("Invalid tag: {}", tag)
                file.insert(0, tag);
                return Some(pool);
            }
        });
    }
    return Some(pool);
}

fn parse_modified_utf8(file: &mut Vec<u8>) -> Option<String>{
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

fn resolve_constants(raw_pool: Vec<RawConstantEntry>) -> Vec<ConstantEntry>{
    let mut ret: Vec<ConstantEntry> = Vec::with_capacity(raw_pool.len());
    for con in raw_pool {
        ret.push(match con {
            RawConstantEntry::Utf8(s) => ConstantEntry::Utf8(box s),
            RawConstantEntry::Integer(i) => ConstantEntry::Integer(i),
            RawConstantEntry::Float(f) => ConstantEntry::Float(f),
            RawConstantEntry::Long(l) => ConstantEntry::Long(l),
            RawConstantEntry::Double(d) => ConstantEntry::Double(d),

            RawConstantEntry::Class(idx) if let RawConstantEntry::Utf8(s) = raw_pool[idx as usize]
                => ConstantEntry::Class(box s),
            RawConstantEntry::StringConst(idx) if let RawConstantEntry::Utf8(s) = raw_pool[idx as usize]
                => ConstantEntry::StringConst(box s),
            RawConstantEntry::MethodType(idx) if let RawConstantEntry::Utf8(s) = raw_pool[idx as usize]
                => ConstantEntry::MethodType(box s),
            RawConstantEntry::Module(idx) if let RawConstantEntry::Utf8(s) = raw_pool[idx as usize]
                => ConstantEntry::Module(box s),
            RawConstantEntry::Package(idx) if let RawConstantEntry::Utf8(s) = raw_pool[idx as usize]
                => ConstantEntry::Package(box s),

            _ => panic!("bad conversion from {:?}", con)
        });
    }
    return ret;
}



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