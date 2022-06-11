
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
// ...