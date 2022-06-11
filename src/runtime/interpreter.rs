use parser::classfile_structs::{Code, MethodInfo, Instruction};
use runtime::jvalue::JValue;

use crate::parser::classfile_structs::ConstantEntry;

pub fn interpret(method: &MethodInfo, args: Vec<JValue>, code: &Code) -> Result<Option<JValue>, &'static str>{
    let mut i: usize = 0;
    let mut stack: Vec<JValue> = Vec::with_capacity(code.max_stack as usize);
    let mut locals: Vec<Option<JValue>> = Vec::with_capacity(code.max_locals as usize);
    locals.append(&mut args.iter().cloned().map(Option::Some).collect());
    locals.resize(code.max_locals as usize, None);
    while i < code.bytecode.len(){
        let mut was_jump = false;
        let (idx, instr) = code.bytecode.get(i).expect("in range");
        match instr {
            Instruction::IConst(it) => {
                stack.insert(0, JValue::Int(*it as i32));
            },
            Instruction::LConst(it) => {
                stack.insert(0, JValue::Long(*it as i64));
                stack.insert(1, JValue::Second);
            },

            Instruction::Ldc(c) => match c{
                ConstantEntry::Integer(i) => {
                    stack.insert(0, JValue::Int(*i));
                }
                ConstantEntry::Long(l) => {
                    stack.insert(0, JValue::Long(*l));
                    stack.insert(1, JValue::Second);
                }
                _ => { panic!("Possibly unhandled or invalid constant: {:?}", c) }
            }

            Instruction::IStore(at) => {
                if let Some(JValue::Int(value)) = stack.get(0){
                    let at = *at as usize; // yeah
                    locals = locals.splice(at..at+1, [Some(JValue::Int(*value))]).collect();
                    stack.remove(0);
                }else{
                    return Err("Tried to execute istore without int on top of stack");
                }
            },
            Instruction::LStore(at) => {
                if let Some(JValue::Long(value)) = stack.get(0){
                    let at = *at as usize;
                    locals = locals.splice(at..at+2, [Some(JValue::Long(*value)), Some(JValue::Second)]).collect();
                    stack.remove(0); stack.remove(0); // get rid of the Second too
                }else{
                    return Err("Tried to execute lstore without long on top of stack");
                }
            },

            Instruction::ILoad(at) => {
                if let Some(Some(JValue::Int(value))) = locals.get(*at as usize){
                    stack.insert(0, JValue::Int(*value));
                }else{
                    return Err("Tried to execute iload without int at local variable index");
                }
            },
            Instruction::LLoad(at) => {
                if let Some(Some(JValue::Long(value))) = locals.get(*at as usize){
                    stack.insert(0, JValue::Long(*value));
                    stack.insert(1, JValue::Second);
                }else{
                    return Err("Tried to execute lload without long at local variable index");
                }
            },

            Instruction::Goto(offset) => {
                let target = (*idx as isize) + (*offset as isize);
                if target < 0{
                    panic!("Bad goto offset");
                }
                i = target as usize;
                was_jump = true;
            },
            Instruction::Goto(offset) => {
                let target = (*idx as isize) + (*offset as isize);
                if target < 0{
                    panic!("Bad goto offset");
                }
                i = target as usize;
                was_jump = true;
            },

            Instruction::IReturn => {
                return if let Some(JValue::Int(ret)) = stack.get(0){
                    Ok(Some(JValue::Int(*ret)))
                }else{
                    Err("Tried to execute ireturn without int on top of stack")
                }
            },
            Instruction::LReturn => {
                return if let Some(JValue::Long(ret)) = stack.get(0){
                    Ok(Some(JValue::Long(*ret)))
                }else{
                    Err("Tried to execute lreturn without long on top of stack")
                }
            },
            Instruction::Return => return Ok(None),

            other => {
                panic!("Unhandled instruction: {:?}", other);
            }
        };
        if !was_jump{
            i += 1;
        }
    }
    return Err("Reached end of function without return!");
}

