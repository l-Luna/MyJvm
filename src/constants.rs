
// Shared flags
pub const ACC_PUBLIC: u16          = 0x0001;
pub const ACC_PRIVATE: u16         = 0x0002;
pub const ACC_PROTECTED: u16       = 0x0004;
pub const ACC_STATIC: u16          = 0x0008;
pub const ACC_FINAL: u16           = 0x0010;
pub const ACC_ABSTRACT: u16        = 0x0400;
pub const ACC_SYNTHETIC: u16       = 0x1000;
pub const ACC_ENUM: u16            = 0x4000;

// Class flags
pub const CLASS_ACC_SUPER: u16         = 0x0020;
pub const CLASS_ACC_INTERFACE: u16     = 0x0200;
pub const CLASS_ACC_ANNOTATION: u16    = 0x2000;
pub const CLASS_ACC_MODULE: u16        = 0x8000;

// Field flags
pub const FIELD_ACC_VOLATILE: u16      = 0x0040;
pub const FIELD_ACC_TRANSIENT: u16     = 0x0080;

pub fn bit_set(flags: u16, flag: u16) -> bool{
    return (flags & flag) == flag;
}

// Bytecode instructions
// Roughly the same order as classfile_structs::Instruction
pub const OP_ICONST_M1: u8              = 2;
pub const OP_ICONST_0: u8               = 3;
pub const OP_ICONST_1: u8               = 4;
pub const OP_ICONST_2: u8               = 5;
pub const OP_ICONST_3: u8               = 6;
pub const OP_ICONST_4: u8               = 7;
pub const OP_ICONST_5: u8               = 8;
pub const OP_BIPUSH: u8                 = 16;

pub const OP_LDC: u8                    = 18;
pub const OP_LDC_W: u8                  = 19;
pub const OP_LDC2_W: u8                 = 20;

pub const OP_IADD: u8                   = 96;

pub const OP_GOTO: u8                   = 167;
pub const OP_GOTO_W: u8                 = 200;
pub const OP_IF_ICMPEQ: u8              = 159;
pub const OP_IF_ICMPNE: u8              = 160;
pub const OP_IF_ICMPLT: u8              = 161;
pub const OP_IF_ICMPGE: u8              = 162;
pub const OP_IF_ICMPGT: u8              = 163;
pub const OP_IF_ICMPLE: u8              = 164;

pub const OP_IRETURN: u8                = 172;
// ...