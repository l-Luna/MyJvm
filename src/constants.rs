
// Shared flags
const ACC_PUBLIC: u16          = 0x0001;
const ACC_PRIVATE: u16         = 0x0002;
const ACC_PROTECTED: u16       = 0x0004;
const ACC_const: u16          = 0x0008;
const ACC_FINAL: u16           = 0x0010;
const ACC_ABSTRACT: u16        = 0x0400;
const ACC_SYNTHETIC: u16       = 0x1000;
const ACC_ENUM: u16            = 0x4000;

// Class flags
const CLASS_ACC_SUPER: u16         = 0x0020;
const CLASS_ACC_INTERFACE: u16     = 0x0200;
const CLASS_ACC_ANNOTATION: u16    = 0x2000;
const CLASS_ACC_MODULE: u16        = 0x8000;

// Field flags
const FIELD_ACC_VOLATILE: u16      = 0x4000;
const FIELD_ACC_TRANSIENTE: u16    = 0x4000;

pub fn bit_set(flags: u16, flag: u16) -> bool{
    return (flags & flag) == flag;
}



pub fn check_class_flags(flags: u16) -> Result<(), &'static str>{
    if bit_set(flags, CLASS_ACC_INTERFACE){
        if !bit_set(flags, ACC_ABSTRACT){
            return Err("Interface class must be abstract");
        }
        if bit_set(flags, ACC_FINAL){
            return Err("Interface class must not be final");
        }
        if bit_set(flags, CLASS_ACC_SUPER){
            return Err("Interface class must not have \"super\" flag");
        }
        if bit_set(flags, ACC_ENUM){
            return Err("Enum class must not be marked as interface");
        }
        if bit_set(flags, CLASS_ACC_MODULE){
            return Err("Module info classfile must not be marked as interface");
        }
    }else{
        if bit_set(flags, CLASS_ACC_ANNOTATION){
            return Err("Annotation class must be marked as interface");
        }
    }
    if bit_set(flags, ACC_ABSTRACT) && bit_set(flags, ACC_FINAL){
        return Err("Class cannot be both abstract and final");
    }
    // TODO: check modules have no other flags
    return Ok(());
}