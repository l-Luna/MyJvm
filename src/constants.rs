
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
pub const OP_NOP: u8                    = 0;

pub const OP_ACONST_NULL: u8            = 1;

pub const OP_ICONST_M1: u8              = 2;
pub const OP_ICONST_0: u8               = 3;
pub const OP_ICONST_1: u8               = 4;
pub const OP_ICONST_2: u8               = 5;
pub const OP_ICONST_3: u8               = 6;
pub const OP_ICONST_4: u8               = 7;
pub const OP_ICONST_5: u8               = 8;
pub const OP_BIPUSH: u8                 = 16;

pub const OP_LCONST_0: u8               = 9;
pub const OP_LCONST_1: u8               = 10;

pub const OP_LDC: u8                    = 18;
pub const OP_LDC_W: u8                  = 19;
pub const OP_LDC2_W: u8                 = 20;

pub const OP_ISTORE: u8                 = 54;
pub const OP_LSTORE: u8                 = 55;
pub const OP_ASTORE: u8                 = 58;

pub const OP_ISTORE_0: u8                 = 59;
pub const OP_ISTORE_1: u8                 = 60;
pub const OP_ISTORE_2: u8                 = 61;
pub const OP_ISTORE_3: u8                 = 62;

pub const OP_LSTORE_0: u8                 = 63;
pub const OP_LSTORE_1: u8                 = 64;
pub const OP_LSTORE_2: u8                 = 65;
pub const OP_LSTORE_3: u8                 = 66;

pub const OP_ASTORE_0: u8                 = 75;
pub const OP_ASTORE_1: u8                 = 76;
pub const OP_ASTORE_2: u8                 = 77;
pub const OP_ASTORE_3: u8                 = 78;

pub const OP_ILOAD: u8                   = 21;
pub const OP_LLOAD: u8                   = 22;
pub const OP_ALOAD: u8                   = 25;

pub const OP_ILOAD_0: u8                 = 26;
pub const OP_ILOAD_1: u8                 = 27;
pub const OP_ILOAD_2: u8                 = 28;
pub const OP_ILOAD_3: u8                 = 29;

pub const OP_LLOAD_0: u8                 = 30;
pub const OP_LLOAD_1: u8                 = 31;
pub const OP_LLOAD_2: u8                 = 32;
pub const OP_LLOAD_3: u8                 = 33;

pub const OP_ALOAD_0: u8                 = 42;
pub const OP_ALOAD_1: u8                 = 43;
pub const OP_ALOAD_2: u8                 = 44;
pub const OP_ALOAD_3: u8                 = 45;

pub const OP_IADD: u8                   = 96;
pub const OP_LADD: u8                   = 97;
pub const OP_ISUB: u8                   = 100;
pub const OP_LSUB: u8                   = 101;
pub const OP_IINC: u8                   = 132;

pub const OP_GOTO: u8                   = 167;
pub const OP_GOTO_W: u8                 = 200;
pub const OP_IF_EQ: u8                  = 153;
pub const OP_IF_NE: u8                  = 154;
pub const OP_IF_LT: u8                  = 155;
pub const OP_IF_GE: u8                  = 156;
pub const OP_IF_GT: u8                  = 157;
pub const OP_IF_LE: u8                  = 158;
pub const OP_IF_ICMP_EQ: u8             = 159;
pub const OP_IF_ICMP_NE: u8             = 160;
pub const OP_IF_ICMP_LT: u8             = 161;
pub const OP_IF_ICMP_GE: u8             = 162;
pub const OP_IF_ICMP_GT: u8             = 163;
pub const OP_IF_ICMP_LE: u8             = 164;

pub const OP_I2L: u8                    = 133;
pub const OP_L2I: u8                    = 136;

pub const OP_IRETURN: u8                = 172;
pub const OP_LRETURN: u8                = 173;
pub const OP_RETURN: u8                 = 177;

pub const OP_GET_STATIC: u8             = 178;
pub const OP_PUT_STATIC: u8             = 179;
pub const OP_GET_FIELD: u8              = 180;
pub const OP_PUT_FIELD: u8              = 181;

pub const OP_INVOKE_VIRTUAL: u8         = 182;
pub const OP_INVOKE_SPECIAL: u8         = 183;
pub const OP_INVOKE_STATIC: u8          = 184;
pub const OP_INVOKE_INTERFACE: u8       = 185;
// ...

// Misc
pub const BOOTSTRAP_LOADER_NAME: &str   = "java.lang.ClassLoader";