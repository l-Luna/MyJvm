use parser::classfile_structs::NameAndType;

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

// Method flags
pub const METHOD_ACC_SYNCHRONIZED: u16 = 0x0100;
pub const METHOD_ACC_NATIVE: u16       = 0x0100;

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
pub const OP_SIPUSH: u8                 = 17;

pub const OP_LCONST_0: u8               = 9;
pub const OP_LCONST_1: u8               = 10;

pub const OP_FCONST_0: u8               = 11;
pub const OP_FCONST_1: u8               = 12;
pub const OP_FCONST_2: u8               = 13;

pub const OP_DCONST_0: u8               = 14;
pub const OP_DCONST_1: u8               = 15;

pub const OP_LDC: u8                    = 18;
pub const OP_LDC_W: u8                  = 19;
pub const OP_LDC2_W: u8                 = 20;

pub const OP_ISTORE: u8                 = 54;
pub const OP_LSTORE: u8                 = 55;
pub const OP_FSTORE: u8                 = 56;
pub const OP_DSTORE: u8                 = 57;
pub const OP_ASTORE: u8                 = 58;

pub const OP_ISTORE_0: u8               = 59;
pub const OP_ISTORE_1: u8               = 60;
pub const OP_ISTORE_2: u8               = 61;
pub const OP_ISTORE_3: u8               = 62;

pub const OP_LSTORE_0: u8               = 63;
pub const OP_LSTORE_1: u8               = 64;
pub const OP_LSTORE_2: u8               = 65;
pub const OP_LSTORE_3: u8               = 66;

pub const OP_FSTORE_0: u8               = 67;
pub const OP_FSTORE_1: u8               = 68;
pub const OP_FSTORE_2: u8               = 69;
pub const OP_FSTORE_3: u8               = 70;

pub const OP_DSTORE_0: u8               = 71;
pub const OP_DSTORE_1: u8               = 72;
pub const OP_DSTORE_2: u8               = 73;
pub const OP_DSTORE_3: u8               = 74;

pub const OP_ASTORE_0: u8               = 75;
pub const OP_ASTORE_1: u8               = 76;
pub const OP_ASTORE_2: u8               = 77;
pub const OP_ASTORE_3: u8               = 78;

pub const OP_IASTORE: u8                = 79;
pub const OP_LASTORE: u8                = 80;
pub const OP_FASTORE: u8                = 81;
pub const OP_DASTORE: u8                = 82;
pub const OP_AASTORE: u8                = 83;
pub const OP_BASTORE: u8                = 84;
pub const OP_CASTORE: u8                = 85;
pub const OP_SASTORE: u8                = 86;

pub const OP_ILOAD: u8                  = 21;
pub const OP_LLOAD: u8                  = 22;
pub const OP_FLOAD: u8                  = 23;
pub const OP_DLOAD: u8                  = 24;
pub const OP_ALOAD: u8                  = 25;

pub const OP_ILOAD_0: u8                = 26;
pub const OP_ILOAD_1: u8                = 27;
pub const OP_ILOAD_2: u8                = 28;
pub const OP_ILOAD_3: u8                = 29;

pub const OP_LLOAD_0: u8                = 30;
pub const OP_LLOAD_1: u8                = 31;
pub const OP_LLOAD_2: u8                = 32;
pub const OP_LLOAD_3: u8                = 33;

pub const OP_FLOAD_0: u8                = 34;
pub const OP_FLOAD_1: u8                = 35;
pub const OP_FLOAD_2: u8                = 36;
pub const OP_FLOAD_3: u8                = 37;

pub const OP_DLOAD_0: u8                = 38;
pub const OP_DLOAD_1: u8                = 39;
pub const OP_DLOAD_2: u8                = 40;
pub const OP_DLOAD_3: u8                = 41;

pub const OP_ALOAD_0: u8                = 42;
pub const OP_ALOAD_1: u8                = 43;
pub const OP_ALOAD_2: u8                = 44;
pub const OP_ALOAD_3: u8                = 45;

pub const OP_IALOAD: u8                  = 46;
pub const OP_LALOAD: u8                  = 47;
pub const OP_FALOAD: u8                  = 48;
pub const OP_DALOAD: u8                  = 49;
pub const OP_AALOAD: u8                  = 50;
pub const OP_BALOAD: u8                  = 51;
pub const OP_CALOAD: u8                  = 52;
pub const OP_SALOAD: u8                  = 53;

pub const OP_POP: u8                     = 87;
pub const OP_POP2: u8                    = 88;
pub const OP_DUP: u8                     = 89;
pub const OP_DUP_X1: u8                  = 90;
pub const OP_DUP_X2: u8                  = 91;
pub const OP_DUP2: u8                    = 92;
pub const OP_DUP2_X1: u8                 = 93;
pub const OP_DUP2_X2: u8                 = 94;
pub const OP_SWAP: u8                    = 95;

pub const OP_IADD: u8                    = 96;
pub const OP_LADD: u8                    = 97;
pub const OP_FADD: u8                    = 98;
pub const OP_DADD: u8                    = 99;

pub const OP_ISUB: u8                    = 100;
pub const OP_LSUB: u8                    = 101;
pub const OP_FSUB: u8                    = 102;
pub const OP_DSUB: u8                    = 103;

pub const OP_IMUL: u8                    = 104;
pub const OP_LMUL: u8                    = 105;
pub const OP_FMUL: u8                    = 106;
pub const OP_DMUL: u8                    = 107;

pub const OP_IDIV: u8                    = 108;
pub const OP_LDIV: u8                    = 109;
pub const OP_FDIV: u8                    = 110;
pub const OP_DDIV: u8                    = 111;

pub const OP_IREM: u8                    = 112;
pub const OP_LREM: u8                    = 113;
pub const OP_FREM: u8                    = 114;
pub const OP_DREM: u8                    = 115;

pub const OP_INEG: u8                    = 116;
pub const OP_LNEG: u8                    = 117;
pub const OP_FNEG: u8                    = 118;
pub const OP_DNEG: u8                    = 119;

pub const OP_ISHL: u8                    = 120;
pub const OP_LSHL: u8                    = 121;
pub const OP_ISHR: u8                    = 122;
pub const OP_LSHR: u8                    = 123;
pub const OP_IUSHR: u8                   = 124;
pub const OP_LUSHR: u8                   = 125;
pub const OP_IAND: u8                    = 126;
pub const OP_LAND: u8                    = 127;
pub const OP_IOR: u8                     = 128;
pub const OP_LOR: u8                     = 129;
pub const OP_IXOR: u8                    = 130;
pub const OP_LXOR: u8                    = 131;

pub const OP_IINC: u8                    = 132;

pub const OP_GOTO: u8                    = 167;
// jsr, ret, not supported
pub const OP_TABLE_SWITCH: u8            = 170;
pub const OP_LOOKUP_SWITCH: u8           = 171;
pub const OP_GOTO_W: u8                  = 200;

pub const OP_LCMP: u8                    = 148;
pub const OP_FCMPL: u8                   = 149;
pub const OP_FCMPG: u8                   = 150;
pub const OP_DCMPL: u8                   = 151;
pub const OP_DCMPG: u8                   = 152;

pub const OP_IF_EQ: u8                   = 153;
pub const OP_IF_NE: u8                   = 154;
pub const OP_IF_LT: u8                   = 155;
pub const OP_IF_GE: u8                   = 156;
pub const OP_IF_GT: u8                   = 157;
pub const OP_IF_LE: u8                   = 158;

pub const OP_IF_ICMP_EQ: u8              = 159;
pub const OP_IF_ICMP_NE: u8              = 160;
pub const OP_IF_ICMP_LT: u8              = 161;
pub const OP_IF_ICMP_GE: u8              = 162;
pub const OP_IF_ICMP_GT: u8              = 163;
pub const OP_IF_ICMP_LE: u8              = 164;

pub const OP_IF_ACMP_EQ: u8              = 165;
pub const OP_IF_ACMP_NE: u8              = 166;

pub const OP_I2L: u8                     = 133;
pub const OP_I2F: u8                     = 134;
pub const OP_I2D: u8                     = 135;

pub const OP_L2I: u8                     = 136;
pub const OP_L2F: u8                     = 137;
pub const OP_L2D: u8                     = 138;

pub const OP_F2I: u8                     = 139;
pub const OP_F2L: u8                     = 140;
pub const OP_F2D: u8                     = 141;

pub const OP_D2I: u8                     = 142;
pub const OP_D2L: u8                     = 143;
pub const OP_D2F: u8                     = 144;

pub const OP_I2B: u8                     = 145;
pub const OP_I2C: u8                     = 146;
pub const OP_I2S: u8                     = 147;

pub const OP_IRETURN: u8                 = 172;
pub const OP_LRETURN: u8                 = 173;
pub const OP_FRETURN: u8                 = 174;
pub const OP_DRETURN: u8                 = 175;
pub const OP_ARETURN: u8                 = 176;
pub const OP_RETURN: u8                  = 177;

pub const OP_GET_STATIC: u8              = 178;
pub const OP_PUT_STATIC: u8              = 179;
pub const OP_GET_FIELD: u8               = 180;
pub const OP_PUT_FIELD: u8               = 181;

pub const OP_INVOKE_VIRTUAL: u8          = 182;
pub const OP_INVOKE_SPECIAL: u8          = 183;
pub const OP_INVOKE_STATIC: u8           = 184;
pub const OP_INVOKE_INTERFACE: u8        = 185;
pub const OP_INVOKE_DYNAMIC: u8          = 186;

pub const OP_NEW: u8                     = 187;
pub const OP_NEWARRAY: u8                = 188;
pub const OP_ANEWARRAY: u8               = 189;
pub const OP_ARRAY_LENGTH: u8            = 190;
pub const OP_ATHROW: u8                  = 191;
pub const OP_CHECK_CAST: u8              = 192;
pub const OP_INSTANCE_OF: u8             = 193;
pub const OP_MONITOR_ENTER: u8           = 194;
pub const OP_MONITOR_EXIT: u8            = 195;

pub const OP_WIDE: u8                    = 196;
pub const OP_MULTI_ANEWARRAY: u8         = 197;
pub const OP_IF_NULL: u8                 = 198;
pub const OP_IF_NONNULL: u8              = 199;
// jsr_w, not supported

pub const OP_BREAKPOINT: u8              = 202;
pub const OP_FREE_1: u8                  = 254;
pub const OP_FREE_2: u8                  = 255;

// Descriptors
pub const CL_INIT_NAME: &str            = "<clinit>";
pub const CL_INIT_DESC: &str            = "()V";

pub const SYSTEM_INIT_1_NAME: &str      = "initPhase1";
pub const SYSTEM_INIT_1_DESC: &str      = "()V";

pub fn clinit() -> NameAndType{
    return NameAndType{
        name: CL_INIT_NAME.to_owned(),
        descriptor: CL_INIT_DESC.to_owned()
    };
}

pub fn system_init_phase_1() -> NameAndType{
    return NameAndType{
        name: SYSTEM_INIT_1_NAME.to_owned(),
        descriptor: SYSTEM_INIT_1_DESC.to_owned()
    };
}

// Misc
pub const BOOTSTRAP_LOADER_NAME: &str   = "java.lang.ClassLoader";