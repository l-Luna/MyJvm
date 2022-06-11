use parser::classfile_structs::{Code, MethodInfo, Instruction};
use runtime::jvalue::JValue;

pub fn interpret(method: &MethodInfo, args: Vec<JValue>, code: &Code) -> Result<Option<JValue>, &'static str>{
    let mut i: usize = 0;
    let mut stack: Vec<JValue> = Vec::with_capacity(code.max_stack as usize);
    while i < code.bytecode.len(){
        let mut was_jump = false;
        let (idx, instr) = code.bytecode.get(i).expect("in range");
        match instr {
            Instruction::IConst(it) => {
                stack.insert(0, JValue::Int(*it as i32));
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
            _ => {
                panic!("Unhandled instruction");
            }
        };
        if !was_jump{
            i += 1;
        }
    }
    return Err("Reached end of function without return!");
}