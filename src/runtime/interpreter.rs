use std::ops::Deref;

use parser::classfile_structs::{Code, Instruction};
use runtime::jvalue::JValue;
use runtime::{native_impls, objects};
use runtime::class::Class;

use crate::parser::classfile_structs::{ConstantEntry, MemberRef};

use super::{jvalue::JObjectData, class::{self, Method, MaybeClass}, heap::{self, JRef}};

#[derive(Debug)]
pub enum MethodResult{
    FinishWithValue(JValue),
    Finish,
    Throw(StackTrace), // TODO: just use JRef
    MachineError(&'static str) // TODO: replace with panics after classfile verification works
}

// Stack traces

#[derive(Debug, Clone)]
pub struct StackTrace(Vec<StackTraceEntry>);

impl StackTrace{
    pub fn new() -> Self{
        return StackTrace(Vec::new());
    }
}

impl std::ops::Deref for StackTrace{
    type Target = Vec<StackTraceEntry>;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

impl std::ops::DerefMut for StackTrace{
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.0;
    }
}

impl std::fmt::Display for StackTrace{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        for entry in self.iter().rev(){
            write!(f, "\tat {}{}", entry.class_name, entry.method_name)?;
            if let Some(l) = entry.line_number{
                write!(f, ":{}", l)?;
            }
            write!(f, "\n")?;
        }
        return Ok(());
    }
}

#[derive(Debug, Clone)]
pub struct StackTraceEntry{
    class_name: String,
    method_name: String,
    line_number: Option<u16>
}

impl StackTraceEntry {
    pub fn new(class_name: String, method_name: String, line_number: Option<u16>) -> Self{
        return Self{ class_name, method_name, line_number };
    }
}

// Method execution

pub fn execute(owner: &Class, method: &Method, args: Vec<JValue>, trace: StackTrace) -> MethodResult{
    println!("Executing {} in {}", &method.name, &owner.name);
    match &method.code{
        class::MethodImpl::Bytecode(bytecode) => interpret(owner, method, args, bytecode, trace),
        class::MethodImpl::Native => native_impls::run_builtin_native(&owner.name, &format!("{}{}", method.name, method.descriptor()), args),
        class::MethodImpl::Abstract => todo!(),
    }
}

pub fn interpret(owner: &Class, method: &Method, args: Vec<JValue>, code: &Code, trace: StackTrace) -> MethodResult{
    let mut i: usize = 0;
    let mut stack: Vec<JValue> = Vec::with_capacity(code.max_stack as usize);
    let mut locals: Vec<Option<JValue>> = Vec::with_capacity(code.max_locals as usize);
    locals.append(&mut args.iter().cloned().map(Some).collect());
    locals.resize(code.max_locals as usize, None);
    while i < code.bytecode.len(){
        let mut was_jump = false;
        let (idx, instr) = code.bytecode.get(i).expect("in range");
        match instr {
            Instruction::AConstNull => {
                stack.insert(0, JValue::Reference(None));
            },

            Instruction::IConst(it) => {
                stack.insert(0, JValue::Int(*it as i32));
            },
            Instruction::LConst(it) => {
                stack.insert(0, JValue::Long(*it as i64));
                stack.insert(1, JValue::Second);
            },
            Instruction::FConst(it) => {
                stack.insert(0, JValue::Float(*it as f32));
            },
            Instruction::DConst(it) => {
                stack.insert(0, JValue::Double(*it as f64));
                stack.insert(1, JValue::Second);
            },

            Instruction::Ldc(c) => match c{
                ConstantEntry::Integer(i) => {
                    stack.insert(0, JValue::Int(*i));
                },
                ConstantEntry::Long(l) => {
                    stack.insert(0, JValue::Long(*l));
                    stack.insert(1, JValue::Second);
                },
                ConstantEntry::Float(f) => {
                    stack.insert(0, JValue::Float(*f));
                },
                ConstantEntry::Double(d) => {
                    stack.insert(0, JValue::Double(*d));
                    stack.insert(1, JValue::Second);
                },
                ConstantEntry::StringConst(s) => {
                    stack.insert(0, heap::add_ref(objects::synthesize_string(&s)));
                },
                ConstantEntry::Class(s) => {
                    stack.insert(0, heap::add_ref(objects::synthesize_class(&s)));
                },
                _ => { panic!("Possibly unhandled or invalid constant: {:?}", c) }
            }

            Instruction::IStore(at) => {
                if let Some(JValue::Int(value)) = stack.get(0){
                    let at = *at as usize;
                    set_and_pad(&mut locals, at, Some(JValue::Int(*value)), None);
                    stack.remove(0);
                }else{
                    return MethodResult::MachineError("Tried to execute istore without int on top of stack");
                }
            },
            Instruction::LStore(at) => {
                if let Some(JValue::Long(value)) = stack.get(0){
                    let at = *at as usize;
                    set_and_pad(&mut locals, at, Some(JValue::Long(*value)), None);
                    set_and_pad(&mut locals, at + 1, Some(JValue::Second), None);
                    stack.remove(0); stack.remove(0); // get rid of the Second too
                }else{
                    return MethodResult::MachineError("Tried to execute lstore without long on top of stack");
                }
            },
            Instruction::FStore(at) => {
                if let Some(JValue::Float(value)) = stack.get(0){
                    let at = *at as usize;
                    set_and_pad(&mut locals, at, Some(JValue::Float(*value)), None);
                    stack.remove(0);
                }else{
                    return MethodResult::MachineError("Tried to execute fstore without float on top of stack");
                }
            },
            Instruction::DStore(at) => {
                if let Some(JValue::Double(value)) = stack.get(0){
                    let at = *at as usize;
                    set_and_pad(&mut locals, at, Some(JValue::Double(*value)), None);
                    set_and_pad(&mut locals, at + 1, Some(JValue::Second), None);
                    stack.remove(0); stack.remove(0); // get rid of the Second too
                }else{
                    return MethodResult::MachineError("Tried to execute dstore without double on top of stack");
                }
            },
            Instruction::AStore(at) => {
                if let Some(JValue::Reference(value)) = stack.get(0){
                    let at = *at as usize;
                    set_and_pad(&mut locals, at, Some(JValue::Reference(*value)), None);
                    stack.remove(0);
                }else{
                    return MethodResult::MachineError("Tried to execute astore without reference on top of stack");
                }
            },

            Instruction::BAStore => {
                if let Some(JValue::Reference(array_ref)) = stack.get(2)
                && let Some(JValue::Int(array_idx)) = stack.get(1)
                && let Some(JValue::Int(value)) = stack.get(0){
                    let array_ref: &Option<JRef> = array_ref; // fix IDE highlighting
                    if let Some(array_ref) = array_ref{
                        let array = array_ref.deref();
                        if let Ok(mut write) = array.data.write(){
                            if let JObjectData::Array(size, values) = &mut *write{
                                if *array_idx < 0 || *array_idx >= (*size as i32){
                                    return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                                }
                                let idx = *array_idx as usize;
                                set_and_pad(values, idx, JValue::Int(to_byte(*value)), JValue::Int(0));
                            }
                        }; //ah. fun.
                    }else{
                        return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                    }

                    stack.remove(0); stack.remove(0); stack.remove(0);
                }else{
                    return MethodResult::MachineError("Tried to execute bastore without array & index & value on top of stack");
                }
            },
            Instruction::CAStore => {
                if let Some(JValue::Reference(array_ref)) = stack.get(2)
                && let Some(JValue::Int(array_idx)) = stack.get(1)
                && let Some(JValue::Int(value)) = stack.get(0){
                    let array_ref: &Option<JRef> = array_ref; // fix IDE highlighting
                    if let Some(array_ref) = array_ref{
                        let array = array_ref.deref();
                        if let Ok(mut write) = array.data.write(){
                            if let JObjectData::Array(size, values) = &mut *write{
                                if *array_idx < 0 || *array_idx >= (*size as i32){
                                    return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                                }
                                let idx = *array_idx as usize;
                                set_and_pad(values, idx, JValue::Int(to_char(*value)), JValue::Int(0));
                            }
                        };
                    }else{
                        return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                    }

                    stack.remove(0); stack.remove(0); stack.remove(0);
                }else{
                    return MethodResult::MachineError("Tried to execute castore without array & index & value on top of stack");
                }
            },
            Instruction::IAStore => {
                if let Some(JValue::Reference(array_ref)) = stack.get(2)
                && let Some(JValue::Int(array_idx)) = stack.get(1)
                && let Some(JValue::Int(value)) = stack.get(0){
                    let array_ref: &Option<JRef> = array_ref; // fix IDE highlighting
                    if let Some(array_ref) = array_ref{
                        let array = array_ref.deref();
                        if let Ok(mut write) = array.data.write(){
                            if let JObjectData::Array(size, values) = &mut *write{
                                if *array_idx < 0 || *array_idx >= (*size as i32){
                                    return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                                }
                                let idx = *array_idx as usize;
                                set_and_pad(values, idx, JValue::Int(*value), JValue::Int(0));
                            }
                        };
                    }else{
                        return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                    }

                    stack.remove(0); stack.remove(0); stack.remove(0);
                }else{
                    return MethodResult::MachineError("Tried to execute iastore without array & index & value on top of stack");
                }
            },
            Instruction::AAStore => {
                if let Some(JValue::Reference(array_ref)) = stack.get(2)
                && let Some(JValue::Int(array_idx)) = stack.get(1)
                && let Some(JValue::Reference(value)) = stack.get(0){
                    let array_ref: &Option<JRef> = array_ref; // fix IDE highlighting
                    if let Some(array_ref) = array_ref{
                        let array = array_ref.deref();
                        if let Ok(mut write) = array.data.write(){
                            if let JObjectData::Array(size, values) = &mut *write{
                                if *array_idx < 0 || *array_idx >= (*size as i32){
                                    return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                                }
                                let idx = *array_idx as usize;
                                set_and_pad(values, idx, JValue::Reference(*value), JValue::Reference(None));
                            }
                        };
                    }else{
                        return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                    }

                    stack.remove(0); stack.remove(0); stack.remove(0);
                }else{
                    return MethodResult::MachineError("Tried to execute aastore without array & index & value on top of stack");
                }
            },

            Instruction::ILoad(at) => {
                if let Some(Some(JValue::Int(value))) = locals.get(*at as usize){
                    stack.insert(0, JValue::Int(*value));
                }else{
                    println!("Locals: {:?}, params: {:?}", &locals, &args);
                    return MethodResult::MachineError("Tried to execute iload without int at local variable index");
                }
            },
            Instruction::LLoad(at) => {
                if let Some(Some(JValue::Long(value))) = locals.get(*at as usize){
                    stack.insert(0, JValue::Long(*value));
                    stack.insert(1, JValue::Second);
                }else{
                    return MethodResult::MachineError("Tried to execute lload without long at local variable index");
                }
            },
            Instruction::FLoad(at) => {
                if let Some(Some(JValue::Float(value))) = locals.get(*at as usize){
                    stack.insert(0, JValue::Float(*value));
                }else{
                    return MethodResult::MachineError("Tried to execute fload without int at local variable index");
                }
            },
            Instruction::DLoad(at) => {
                if let Some(Some(JValue::Double(value))) = locals.get(*at as usize){
                    stack.insert(0, JValue::Double(*value));
                    stack.insert(1, JValue::Second);
                }else{
                    return MethodResult::MachineError("Tried to execute dload without long at local variable index");
                }
            },
            Instruction::ALoad(at) => {
                if let Some(Some(JValue::Reference(value))) = locals.get(*at as usize){
                    stack.insert(0, JValue::Reference(*value));
                }else{
                    return MethodResult::MachineError("Tried to execute aload without reference at local variable index");
                }
            },

            Instruction::BALoad => {
                if let Some(JValue::Reference(array_ref)) = stack.get(1)
                && let Some(JValue::Int(array_idx)) = stack.get(0){
                    let array_ref: &Option<JRef> = array_ref; // fix IDE highlighting
                    if let Some(array_ref) = array_ref{
                        let array = array_ref.deref();
                        if let Ok(read) = array.data.read(){
                            if let JObjectData::Array(size, values) = &*read{
                                if *array_idx < 0 || *array_idx >= (*size as i32){
                                    return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                                }
                                let idx = *array_idx as usize;
                                stack.insert(0, values[idx].clone());
                            }
                        };
                    }else{
                        return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                    }

                    stack.remove(0); stack.remove(0);
                }else{
                    println!("stack is {:?} in {}", &stack, &method.name);
                    return MethodResult::MachineError("Tried to execute baload without array & index on top of stack");
                }
            },

            Instruction::Dup => {
                if let Some(value) = stack.get(0){
                    stack.insert(0, value.clone());
                }else{
                    return MethodResult::MachineError("Tried to execute dup with empty stack");
                }
            },

            // TODO: merge into one match arm (instr?) and match on the instruction inside
            Instruction::IAdd => {
                if let Some(JValue::Int(l)) = stack.get(0)
                && let Some(JValue::Int(r)) = stack.get(1){
                    let (val, _) = i32::overflowing_add(*l, *r);
                    stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Int(val));
                }else{
                    return MethodResult::MachineError("Tried to execute iadd without two ints on top of stack");
                }
            },
            Instruction::ISub => {
                if let Some(JValue::Int(l)) = stack.get(0)
                && let Some(JValue::Int(r)) = stack.get(1){
                    let (val, _) = i32::overflowing_add(-*l, *r); // value2 - value1
                    stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Int(val));
                }else{
                    return MethodResult::MachineError("Tried to execute isub without two ints on top of stack");
                }
            },
            Instruction::IMul => {
                if let Some(JValue::Int(l)) = stack.get(0)
                && let Some(JValue::Int(r)) = stack.get(1){
                    let val = *l * *r;
                    stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Int(val));
                }else{
                    return MethodResult::MachineError("Tried to execute imul without two ints on top of stack");
                }
            },
            Instruction::IDiv => {
                if let Some(JValue::Int(l)) = stack.get(0)
                && let Some(JValue::Int(r)) = stack.get(1){
                    let val = *l / *r;
                    stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Int(val));
                }else{
                    return MethodResult::MachineError("Tried to execute idiv without two ints on top of stack");
                }
            },
            Instruction::IRem => {
                if let Some(JValue::Int(value2)) = stack.get(0)
                && let Some(JValue::Int(value1)) = stack.get(1){
                    let val = value1 - (value1 / value2) * value2; // JVMS
                    stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Int(val));
                }else{
                    return MethodResult::MachineError("Tried to execute irem without two ints on top of stack");
                }
            },
            Instruction::IShl => {
                if let Some(JValue::Int(value2)) = stack.get(0)
                && let Some(JValue::Int(value1)) = stack.get(1){
                    let val = *value1 << (*value2 & 0b00011111);
                    stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Int(val));
                }else{
                    return MethodResult::MachineError("Tried to execute ishl without two ints on top of stack");
                }
            },
            Instruction::IShr => {
                if let Some(JValue::Int(value2)) = stack.get(0)
                && let Some(JValue::Int(value1)) = stack.get(1){
                    let val = *value1 << (*value2 & 0b00011111);
                    stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Int(val));
                }else{
                    dbg!(&stack);
                    return MethodResult::MachineError("Tried to execute ishr without two ints on top of stack");
                }
            },
            Instruction::IUshr => {
                if let Some(JValue::Int(value2)) = stack.get(0)
                && let Some(JValue::Int(value1)) = stack.get(1){
                    let val = ((*value1 as u32) << ((*value2 & 0b00011111) as u32)) as i32;
                    stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Int(val));
                }else{
                    return MethodResult::MachineError("Tried to execute iushr without two ints on top of stack");
                }
            },
            Instruction::IAnd => {
                if let Some(JValue::Int(l)) = stack.get(0)
                && let Some(JValue::Int(r)) = stack.get(1){
                    let val = *l & *r;
                    stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Int(val));
                }else{
                    return MethodResult::MachineError("Tried to execute iand without two ints on top of stack");
                }
            },

            Instruction::FDiv => {
                if let Some(JValue::Float(l)) = stack.get(0)
                && let Some(JValue::Float(r)) = stack.get(1){
                    let val = *l / *r;
                    stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Float(val));
                }else{
                    return MethodResult::MachineError("Tried to execute fdiv without two floats on top of stack");
                }
            },

            Instruction::DAdd => {
                if let Some(JValue::Double(l)) = stack.get(0)
                && let Some(JValue::Double(r)) = stack.get(2){
                    let val = *l + *r;
                    stack.remove(0); stack.remove(0); stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Double(val));
                    stack.insert(1, JValue::Second);
                }else{
                    return MethodResult::MachineError("Tried to execute dadd without two ints on top of stack");
                }
            },

            Instruction::IInc(at, inc) => {
                if let Some(Some(JValue::Int(value))) = locals.get(*at as usize){
                    let at = *at as usize;
                    let new_value = *value + *inc as i32;
                    set_and_pad(&mut locals, at, Some(JValue::Int(new_value)), None);
                }
            }

            Instruction::Goto(offset) => {
                let target = (*idx as isize) + (*offset as isize);
                if target < 0{
                    panic!("Bad goto offset");
                }
                i = bytecode_idx_to_instr_idx(target as usize, code);
                was_jump = true;
            },
            
            Instruction::LCmp => {
                if let Some(JValue::Long(val2)) = stack.get(0)
                && let Some(JValue::Long(val1)) = stack.get(2){
                    let val = if val1 == val2{ 0 }
                        else if val1 > val2{ 1 }
                        else{ -1 };
                    stack.remove(0); stack.remove(0); stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Int(val));
                }else{
                    return MethodResult::MachineError("Tried to execute lcmp without two longs on top of stack");
                }
            },
            Instruction::FCmpL | Instruction::FCmpG => {
                if let Some(JValue::Float(val2)) = stack.get(0)
                && let Some(JValue::Float(val1)) = stack.get(1){
                    let val = if val1 == val2{ 0 }
                        else if val1 > val2{ 1 }
                        else if val1 < val2{ -1 }
                        else{
                            if *instr == Instruction::FCmpG{ 1 }
                            else{ -1 }
                        };
                    stack.remove(0); stack.remove(0);
                    stack.insert(0, JValue::Int(val));
                }else{
                    return MethodResult::MachineError("Tried to execute fcmp* without two floats on top of stack");
                }
            },
            
            Instruction::IfEq(offset) => {
                if let JValue::Int(value) = stack.remove(0){
                    if value == 0{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute ifeq without int on top of stack");
                }
            },
            Instruction::IfNe(offset) => {
                if let JValue::Int(value) = stack.remove(0){
                    if value != 0{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute ifeq without int on top of stack");
                }
            },
            Instruction::IfLt(offset) => {
                if let JValue::Int(value) = stack.remove(0){
                    if value < 0{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute iflt without int on top of stack");
                }
            },
            Instruction::IfGe(offset) => {
                if let JValue::Int(value) = stack.remove(0){
                    if value >= 0{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute ifge without int on top of stack");
                }
            },
            Instruction::IfGt(offset) => {
                if let JValue::Int(value) = stack.remove(0){
                    if value > 0{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute ifgt without int on top of stack");
                }
            },
            Instruction::IfLe(offset) => {
                if let JValue::Int(value) = stack.remove(0){
                    if value <= 0{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute ifle without int on top of stack");
                }
            },

            Instruction::IfICmpEq(offset) => {
                if let JValue::Int(value2) = stack.remove(0)
                && let JValue::Int(value1) = stack.remove(0){
                    if value1 == value2{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute if_icmpeq without int on top of stack");
                }
            },
            Instruction::IfICmpNe(offset) => {
                if let JValue::Int(value2) = stack.remove(0)
                && let JValue::Int(value1) = stack.remove(0){
                    if value1 != value2{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute if_icmpne without int on top of stack");
                }
            },
            Instruction::IfICmpLt(offset) => {
                if let JValue::Int(value2) = stack.remove(0)
                && let JValue::Int(value1) = stack.remove(0){
                    if value1 < value2{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute if_icmpeq without int on top of stack");
                }
            },
            Instruction::IfICmpGe(offset) => {
                if let JValue::Int(value2) = stack.remove(0)
                && let JValue::Int(value1) = stack.remove(0){
                    if value1 >= value2{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute if_icmpge without int on top of stack");
                }
            },
            Instruction::IfICmpGt(offset) => {
                if let JValue::Int(value2) = stack.remove(0)
                && let JValue::Int(value1) = stack.remove(0){
                    if value1 > value2{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute if_icmpeq without int on top of stack");
                }
            },
            Instruction::IfICmpLe(offset) => {
                if let JValue::Int(value2) = stack.remove(0)
                && let JValue::Int(value1) = stack.remove(0){
                    if value1 <= value2{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute if_icmpeq without int on top of stack");
                }
            },

            Instruction::IfNull(offset) => {
                if let JValue::Reference(r) = stack.remove(0){
                    if let None = r{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute ifnonnull without reference on top of stack");
                }
            },
            Instruction::IfNonnull(offset) => {
                if let JValue::Reference(r) = stack.remove(0){
                    if let Some(_) = r{
                        let target = (*idx as isize) + (*offset as isize);
                        if target < 0{
                            panic!("Bad goto offset");
                        }
                        i = bytecode_idx_to_instr_idx(target as usize, code);
                        was_jump = true;
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute ifnonnull without reference on top of stack");
                }
            },

            Instruction::I2L => {
                if let JValue::Int(i) = stack.remove(0){
                    let val = i as i64;
                    stack.insert(0, JValue::Long(val));
                    stack.insert(1, JValue::Second);
                }else{
                    return MethodResult::MachineError("Tried to execute i2l without int on top of stack");
                }
            },
            Instruction::L2I => {
                if let JValue::Long(l) = stack.remove(0){
                    let val = l as i32;
                    stack.remove(0);
                    stack.insert(0, JValue::Int(val));
                }else{
                    return MethodResult::MachineError("Tried to execute l2i without long on top of stack");
                }
            },
            Instruction::L2F => {
                if let JValue::Long(l) = stack.remove(0){
                    let val = l as f32;
                    stack.remove(0);
                    stack.insert(0, JValue::Float(val));
                }else{
                    return MethodResult::MachineError("Tried to execute l2f without long on top of stack");
                }
            },
            Instruction::F2D => {
                if let JValue::Float(l) = stack.remove(0){
                    let val = l as f64;
                    stack.insert(0, JValue::Double(val));
                    stack.insert(1, JValue::Second);
                }else{
                    return MethodResult::MachineError("Tried to execute f2d without float on top of stack");
                }
            },
            Instruction::D2L => {
                if let JValue::Double(l) = stack.remove(0){
                    let val = l as i64;
                    stack.remove(0);
                    stack.insert(0, JValue::Long(val));
                    stack.insert(1, JValue::Second);
                }else{
                    return MethodResult::MachineError("Tried to execute d2l without float on top of stack");
                }
            },

            Instruction::IReturn => {
                return if let Some(JValue::Int(ret)) = stack.get(0){
                    MethodResult::FinishWithValue(JValue::Int(*ret))
                }else{
                    MethodResult::MachineError("Tried to execute ireturn without int on top of stack")
                }
            },
            Instruction::LReturn => {
                return if let Some(JValue::Long(ret)) = stack.get(0){
                    MethodResult::FinishWithValue(JValue::Long(*ret))
                }else{
                    MethodResult::MachineError("Tried to execute lreturn without long on top of stack")
                }
            },
            Instruction::FReturn => {
                return if let Some(JValue::Float(ret)) = stack.get(0){
                    MethodResult::FinishWithValue(JValue::Float(*ret))
                }else{
                    MethodResult::MachineError("Tried to execute freturn without float on top of stack")
                }
            },
            Instruction::DReturn => {
                return if let Some(JValue::Double(ret)) = stack.get(0){
                    MethodResult::FinishWithValue(JValue::Double(*ret))
                }else{
                    MethodResult::MachineError("Tried to execute dreturn without double on top of stack")
                }
            },
            Instruction::AReturn => {
                return if let Some(JValue::Reference(ret)) = stack.get(0){
                    MethodResult::FinishWithValue(JValue::Reference(*ret))
                }else{
                    MethodResult::MachineError("Tried to execute areturn without reference on top of stack")
                }
            },
            Instruction::Return => return MethodResult::Finish,

            Instruction::GetField(target) => {
                let owner = heap::get_or_create_bt_class(format!("L{};", target.owner_name.clone()))
                    .expect("Could not load field owner")
                    .ensure_loaded()
                    .expect("Could not load field owner");
                let mut was_static = false;
                for f in &owner.static_fields{
                    let f = f.read().unwrap();
                    if f.0.name == target.name_and_type.name{
                        let j_value = f.1.clone();
                        stack.insert(0, j_value);
                        was_static = true;
                    }
                }
                if !was_static{
                    if let JValue::Reference(r) = stack.remove(0){
                        if let Some(r) = r{
                            let obj = r.deref();
                            if let JObjectData::Fields(f) = &*obj.data.read().unwrap(){
                                let mut pushed = false;
                                for (name, value) in f{
                                    if &target.name_and_type.name == name{
                                        stack.insert(0, value.clone());
                                        pushed = true;
                                        break;
                                    }
                                }
                                if !pushed{
                                    // field declared in class but not present in actual fields
                                    // can happen if object is badly made (like `Class`es currently)
                                    stack.insert(0, JValue::default_value_for(&target.name_and_type.descriptor));
                                }
                            }else{
                                return MethodResult::MachineError("Tried to execute getfield on array reference!");
                            };
                        }else{
                            return MethodResult::Throw(update_trace(&trace, *idx, method, &owner));
                        }
                    }else{
                        return MethodResult::MachineError("Tried to execute getfield without reference on stack!");
                    }
                }
            },
            Instruction::PutField(target) => {
                let field_owner = heap::get_or_create_bt_class(format!("L{};", target.owner_name.clone()))
                    .expect("Could not load field owner")
                    .ensure_loaded()
                    .expect("Could not load field owner");
                let mut was_static = false;
                let value = stack.remove(0);
                for f in &field_owner.static_fields{
                    let mut f = f.write().unwrap();
                    if f.0.name == target.name_and_type.name{
                        f.1 = value;
                        was_static = true;
                        break;
                    }
                }
                if !was_static{
                    let object_ref = stack.remove(0);
                    for f in &field_owner.instance_fields{
                        if f.name == target.name_and_type.name{
                            if let JValue::Reference(Some(r)) = object_ref{
                                let object = r.deref();
                                let mut data = object.data.write().unwrap();
                                if let JObjectData::Fields(fields) = &mut *data{
                                    fields.insert(f.name.clone(), value);
                                }else{
                                    return MethodResult::MachineError("Tried to execute putfield on an array reference!");
                                }
                            }else if let JValue::Reference(None) = object_ref{
                                return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                            }else{
                                return MethodResult::MachineError("Tried to execute putfield with non-reference on stack!")
                            }
                        }
                    }
                }
            },
            
            Instruction::InvokeVirtual(target) => {
                let params = resolve_signature(&target);
                let mut args = Vec::with_capacity(params.len() + 1);
                for _ in 0..params.len(){
                    args.insert(0, stack.remove(0));
                }
                let receiver = stack.remove(0);
                args.insert(0, receiver.clone());

                if let JValue::Reference(Some(r)) = receiver{
                    let class = &r.deref().class;
                    let target = class.virtual_method(&target.name_and_type)
                        .expect(format!("Tried to execute invokevirtual for method with {:?} that doesn't exist on receiver of type {} inside {}.{}{}", &target, &r.deref().class.name, &owner.name, &method.name, &method.descriptor()).as_str());
                    let result = execute(&*class, &target, args, update_trace(&trace, *idx, method, owner));
                    // TODO: exception handling
                    match result{
                        MethodResult::FinishWithValue(v) => stack.push(v),
                        MethodResult::Finish => {},
                        MethodResult::Throw(e) => return MethodResult::Throw(e),
                        MethodResult::MachineError(e) => return MethodResult::MachineError(e),
                    }
                }else if let JValue::Reference(None) = receiver{
                    return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                }else{
                    return MethodResult::MachineError("Tried to execute invokevirtual without object on stack");
                }
            },
            Instruction::InvokeStatic(s) => {
                let owner_name = &s.owner_name;
                let class = heap::get_or_create_bt_class(format!("L{};", owner_name)).unwrap().ensure_loaded().unwrap();
                if let Some(target) = class.static_method(&s.name_and_type){
                    // TODO: dedup code
                    let num_params = target.parameters.len();
                    let mut args = Vec::with_capacity(num_params);
                    for _ in 0..num_params{
                        args.insert(0, stack.remove(0));
                    }
                    let result = execute(&*class, &target, args, update_trace(&trace, *idx, method, owner));
                    match result{
                        MethodResult::FinishWithValue(v) => stack.push(v),
                        MethodResult::Finish => {},
                        MethodResult::Throw(e) => return MethodResult::Throw(e),
                        MethodResult::MachineError(e) => return MethodResult::MachineError(e),
                    }
                }else{
                    // TODO: should throw instead?
                    return MethodResult::MachineError("Tried to execute invokestatic for a method that doesn't exist");
                }
            },
            Instruction::InvokeSpecial(target) => {
                let params = resolve_signature(&target);
                let mut args = Vec::with_capacity(params.len() + 1);
                for _ in 0..params.len(){
                    args.insert(0, stack.remove(0));
                }
                let receiver = stack.remove(0);
                args.insert(0, receiver.clone());

                if let JValue::Reference(Some(r)) = receiver{
                    let class = &r.deref().class;
                    let (target, owner) = class.special_method(&target.name_and_type, target.owner_name.clone())
                        .expect(format!("Tried to execute invokespecial for method with {:?} for {} that doesn't exist on receiver", &target.name_and_type, &target.owner_name.clone()).as_str());
                    let result = execute(owner, &target, args, update_trace(&trace, *idx, method, owner));
                    // TODO: exception handling
                    match result{
                        MethodResult::FinishWithValue(v) => stack.push(v),
                        MethodResult::Finish => {},
                        MethodResult::Throw(e) => return MethodResult::Throw(e),
                        MethodResult::MachineError(e) => return MethodResult::MachineError(e),
                    }
                }else if let JValue::Reference(None) = receiver{
                    return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                }else{
                    return MethodResult::MachineError("Tried to execute invokespecial without object on stack");
                }
            },

            Instruction::ArrayLength => {
                if let Some(JValue::Reference(array_ref)) = stack.get(0){
                    if let Some(array_ref) = array_ref{
                        let array = array_ref.deref();
                        if let Ok(read) = array.data.read(){
                            if let JObjectData::Array(size, _) = &*read{
                                stack.insert(0, JValue::Int(*size as i32));
                            }else{
                                return MethodResult::MachineError("Tried to execute arraylength on non-array reference!");
                            }
                        }else{
                            return MethodResult::MachineError("Could not read object data for arraylength");
                        };
                    }else{
                        return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                    }

                    stack.remove(1); // 0 is the length we just pushed
                }else{
                    return MethodResult::MachineError("Tried to execute arraylength without reference on top of stack");
                }
            },

            Instruction::New(class_name) => {
                let class = heap::get_or_create_bt_class(format!("L{};", class_name))
                    .expect("Could not parse class for new instruction!")
                    .ensure_loaded()
                    .expect("Could not link class for new instruction!");
                stack.insert(0, objects::create_new(class));
            },
            Instruction::NewArray(class_name) => {
                let class = heap::get_or_create_bt_class(class_name.clone())
                    .expect("Could not parse class for anewarray instruction!")
                    .ensure_loaded()
                    .expect("Could not link class for anewarray instruction!");
                if let JValue::Int(l) = stack.remove(0){
                    if l < 0{
                        // TODO: synthesize NegativeArraySizeException
                        return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                    }
                    let l = l as usize;
                    stack.insert(0, objects::create_new_array(class, l));
                }
            },

            Instruction::CheckCast(to) => {
                if let Some(v) = stack.get(0){
                    if let JValue::Reference(r) = v{
                        if let Some(r) = r{
                            let class = &r.deref().class;
                            if !class.assignable_to(&to){
                                return MethodResult::Throw(update_trace(&trace, *idx, method, owner));
                            }
                        }
                    }else{
                        return MethodResult::MachineError("Tried to execute checkcast with non-reference on stack!");
                    }
                }else{
                    return MethodResult::MachineError("Tried to execute checkcast with nothing on stack!");
                }
            },

            other => {
                panic!("Unhandled instruction: {:?}", other);
            }
        };
        if !was_jump{
            i += 1;
        }
    }
    return MethodResult::MachineError("Reached end of function without return!");
}

fn bytecode_idx_to_instr_idx(bytecode_idx: usize, code: &Code) -> usize{
    let mut i = 0;
    for (t_bidx, _) in &code.bytecode{
        if *t_bidx == bytecode_idx{
            return i;
        }
        i += 1;
    }
    panic!("invalid bytecode offset {}", bytecode_idx);
}

fn bytecode_idx_to_line_number(bytecode_idx: usize, method: &Method) -> Option<u16>{
    let table = method.line_number_table.as_ref()?;
    for entry in table.iter().rev(){
        if entry.bytecode_idx < (bytecode_idx as u16){
            return Some(entry.line_number);
        }
    }
    return None;
}

fn update_trace(trace: &StackTrace, bytecode_idx: usize, method: &Method, owner: &Class) -> StackTrace{
    let mut trace = trace.clone();
    trace.push(StackTraceEntry::new(
        owner.name.clone(),
        method.name.clone(),
        bytecode_idx_to_line_number(bytecode_idx, method)));
    return trace;
}

fn set_and_pad<T>(list: &mut Vec<T>, idx: usize, value: T, default: T) where T: Clone{
    if list.len() <= idx{
        for _ in 0..(idx - list.len()){
            list.push(default.clone());
        }
        list.push(value);
    }else{
        list.remove(idx);
        list.insert(idx, value);
    }
}

fn resolve_signature(target: &MemberRef) -> Vec<MaybeClass>{
    let owner = heap::get_or_create_bt_class(format!("L{};", target.owner_name.clone()))
        .expect("Could not load field owner")
        .ensure_loaded()
        .expect("Could not load field owner");
    return owner.virtual_method(&target.name_and_type)
        .expect("Tried to invoke method that does not exist")
        .parameters
        .clone();
}

fn to_short(v: i32) -> i32{
    return v.clamp(i16::MIN as i32, i16::MAX as i32);
}

fn to_char(v: i32) -> i32{
    return v.clamp(u16::MIN as i32, u16::MAX as i32);
}

fn to_byte(v: i32) -> i32{
    return v.clamp(i8::MIN as i32, i8::MAX as i32);
}