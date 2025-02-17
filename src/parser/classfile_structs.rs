use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Classfile{
    pub major_ver: u16,
    pub minor_ver: u16,

    pub constants: Vec<ConstantEntry>,

    pub flags: u16,

    pub name: String,
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,

    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<Attribute>
}

// Ordered by tag number
#[derive(Debug, Clone)]
pub enum RawConstantEntry {
    Utf8(String),   // 1, u16 length + u8[len] modified utf8
    Integer(i32),   // 3
    Float(f32),     // 4
    Long(i64),      // 5, ***uses two entries***
    Double(f64),    // 6, ***uses two entries***, specially handles NaNs

    LongSecond,

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

// Ordered by tag number, uses actual useful values
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantEntry{
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),

    LongSecond,

    Class(String),
    StringConst(String),
    MemberRef(MemberRef),
    NameAndType(NameAndType),
    MethodHandle(DynamicReferenceType, MemberRef),
    MethodType(String),
    Dynamic(Dynamic),
    InvokeDynamic(Dynamic),

    Module(String),
    Package(String)
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemberKind {
    Field, Method, InterfaceMethod
}
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicReferenceType{
    GetField, GetStatic, PutField, PutStatic, InvokeVirtual, InvokeStatic, InvokeSpecial, NewInvokeSpecial, InvokeInterface
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameAndType{ pub name: String, pub descriptor: String }
#[derive(Debug, Clone, PartialEq)]
pub struct MemberRef{ pub kind: MemberKind, pub owner_name: String, pub name_and_type: NameAndType }
#[derive(Debug, Clone, PartialEq)]
pub struct Dynamic{ pub bootstrap: NameAndType, pub value: NameAndType }

#[derive(Debug, Clone, PartialEq)]
pub struct FieldInfo{
    pub flags: u16,
    pub name: String,
    pub desc: String,
    pub attributes: Vec<Attribute>
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodInfo{
    pub flags: u16,
    pub name: String,
    pub desc: Vec<String>,
    pub attributes: Vec<Attribute>
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordComponentInfo{
    pub name: String,
    pub desc: String,
    pub attributes: Vec<Attribute>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Attribute{ // ordered by location
    // Classfile attributes
    SourceFile(String),
    InnerClasses{ /* TODO */ },
    EnclosingMethod{ owner_class: String, owner_method: NameAndType },
    SourceDebugExtension(String),
    BootstrapMethods(Vec<BootstrapEntry>),
    Module{ /* TODO */ }, ModulePackages(Vec<String>), ModuleMainClass(String),
    NestHost(String), NestMembers(Vec<String>),
    Record(Vec<RecordComponentInfo>),
    PermittedSubclasses(Vec<String>),
    
    // Field attributes
    ConstantValue(ConstantEntry),

    // Method attributes
    Code(Code),
    Exceptions(Vec<String>),
    RuntimeVisibleParameterAnnotations(Vec<Vec<Annotation>>),
    RuntimeInvisibleParameterAnnotations(Vec<Vec<Annotation>>),
    AnnotationDefault{ /* TODO */ },
    MethodParameters(Vec<ParameterInfo>),

    // Code attributes
    LineNumberTable(Vec<LineNumberMapping>),
    LocalVariableTable(Vec<LocalVariableEntry>),
    LocalVariableTypeTable(Vec<LocalVariableEntry>),
    StackMapTable{ /* TODO */ },

    // Class, member attributes
    Synthetic, Deprecated, // zero-length
    // Class, member, record component attributes
    Signature(String),
    // Class, member, record component, code attributes
    RuntimeVisibleAnnotations(Vec<Annotation>), RuntimeInvisibleAnnotations(Vec<Annotation>),
    RuntimeVisibleTypeAnnotations{ /* TODO */ }, RuntimeInvisibleTypeAnnotations{ /* TODO */ },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineNumberMapping{
    pub bytecode_idx: u16,
    pub line_number: u16
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalVariableEntry{
    pub start_idx: u16,
    pub end_idx: u16,
    pub name: String,
    pub desc: String,
    pub sig: Option<String>,
    pub lv_idx: u16
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterInfo{
    pub name: Option<String>,
    pub flags: u16
}

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation{
    pub class: String,
    pub data: HashMap<String, Vec<u8>> // TODO: annotation parsing
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapEntry{
    pub ref_type: DynamicReferenceType,
    pub method: MemberRef,
    pub args: Vec<ConstantEntry>
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionHandler{
    pub start_idx: u16,
    pub end_idx: u16,
    pub handler_idx: u16,
    pub catch_type: Option<String>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Code{
    pub max_stack: u16,
    pub max_locals: u16,
    pub bytecode: Vec<(usize, Instruction)>,
    pub exception_handlers: Vec<ExceptionHandler>,
    pub attributes: Vec<Attribute>
}

#[derive(Debug, Clone, PartialEq)]
// only supported ones rn
pub enum Instruction{
    IConst(i32), LConst(i64), FConst(f32), DConst(f64), AConstNull,

    Ldc(ConstantEntry), // copy it here for now

    IStore(u8), LStore(u8), FStore(u8), DStore(u8), AStore(u8),

    IAStore, LAStore, FAStore, DAStore, AAStore, BAStore, CAStore, SAStore,

    ILoad(u8), LLoad(u8), FLoad(u8), DLoad(u8), ALoad(u8),

    IALoad, LALoad, FALoad, DALoad, AALoad, BALoad, CALoad, SALoad,

    Pop, Pop2,
    Dup, DupX1, DupX2,
    Dup2, Dup2X1, Dup2X2,
    Swap,

    IAdd, LAdd, FAdd, DAdd,
    ISub, LSub, FSub, DSub,
    IMul, LMul, FMul, DMul,
    IDiv, LDiv, FDiv, DDiv,
    IRem, LRem, FRem, DRem,
    INeg, LNeg, FNeg, DNeg,

    IShl, LShl, IShr, LShr, IUshr, LUshr,
    IAnd, LAnd, IOr, LOr, IXor, LXor,

    IInc(u8, i8),

    Goto(i32), GotoWide(i64),
    TableSwitch(i32, i32, i32, Vec<i32>), LookupSwitch(i32, Vec<(i32, i32)>),

    LCmp, FCmpL, FCmpG, DCmpL, DCmpG,

    IfEq(i32), IfNe(i32), IfLt(i32), IfGe(i32), IfGt(i32), IfLe(i32),
    IfICmpEq(i32), IfICmpNe(i32), IfICmpLt(i32), IfICmpGe(i32), IfICmpGt(i32), IfICmpLe(i32),
    IfACmpEq(i32), IfACmpNe(i32), IfNull(i32), IfNonnull(i32),

    I2L, I2F, I2D,
    L2I, L2F, L2D,
    F2I, F2L, F2D,
    D2I, D2L, D2F,
    I2B, I2C, I2S,

    IReturn, LReturn, FReturn, DReturn, AReturn, Return, AThrow,

    GetField(MemberRef),
    PutField(MemberRef),

    InvokeVirtual(MemberRef),
    InvokeSpecial(MemberRef),
    InvokeStatic(MemberRef),
    InvokeInterface(MemberRef),
    InvokeDynamic(Dynamic),

    ArrayLength,

    New(String), NewArray(String), MultiANewArray(String, u32),

    CheckCast(String), InstanceOf(String),

    MonitorEnter, MonitorExit,

    TODO
}