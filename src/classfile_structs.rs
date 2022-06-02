
#[derive(Debug)]
pub struct Classfile{
    pub major_ver: u16,
    pub minor_ver: u16,

    pub constants: Vec<RawConstantEntry>,

    pub flags: u16,

    // constant pool indexes (maybe replace with proper references?)
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,

    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<Attribute>
}

// Ordered by tag number
#[derive(Debug)]
pub enum RawConstantEntry {
    Utf8(String),   // 1, u16 length + u8[len] modified utf8
    Integer(i32),   // 3
    Float(f32),     // 4
    Long(i64),      // 5, uses two entries
    Double(f64),    // 6, uses two entries, specially handles NaNs

    Class(u16),     // 7, index to utf8 name
    StringConst(u16),    // 8, index to utf8 content

    MemberRef(u8, u16, u16),    // 9/10/11, tag, index to class, index to name-and-type

    NameAndType(u16, u16),  // 12, index to utf8 name, index to utf8 descriptor

    MethodHandle(u8, u16),  // 15, reference type, index to [field-ref (1-4), method-ref (5-8), iface-method-ref (6/7/9)]
    MethodType(u16),        // 16, index to utf8 descriptor
    Dynamic(u8, u16, u16),  // 17/18, tag, index in bootstrap table, index to name-and-type

    Module(u16),    // 19, index to utf8 name, only in module-info
    Package(u16)    // 20, index to utf8 name, only in module-info
}

#[derive(Debug)]
pub struct Attribute{

}

#[derive(Debug)]
pub struct FieldInfo{

}

#[derive(Debug)]
pub struct MethodInfo{

}