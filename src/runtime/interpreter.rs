use parser::classfile_structs::{Code, Instruction};
use runtime::jvalue::JValue;
use runtime::objects;

use crate::parser::classfile_structs::ConstantEntry;

use super::{class::{Method, self}, heap};

#[derive(Debug)]
pub enum MethodResult{
    FinishWithValue(JValue),
    Finish,
    Throw(JValue),
    MachineError(&'static str) // TODO: replace with panics after classfile verification works
}

pub fn execute(method: &Method, args: Vec<JValue>) -> MethodResult{
    println!("Executing {}", method.name.clone());
    match &method.code{
        class::MethodImpl::Bytecode(bytecode) => interpret(method, args, bytecode),
        class::MethodImpl::Native => todo!(),
        class::MethodImpl::Abstract => todo!(),
    }
}

pub fn interpret(method: &Method, args: Vec<JValue>, code: &Code) -> MethodResult{
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
                ConstantEntry::StringConst(s) => {
                    stack.insert(0, heap::add_ref(objects::synthesize_string(s)));
                }
                _ => { panic!("Possibly unhandled or invalid constant: {:?}", c) }
            }

            Instruction::IStore(at) => {
                if let Some(JValue::Int(value)) = stack.get(0){
                    let at = *at as usize; // yeah
                    locals = locals.splice(at..at+1, [Some(JValue::Int(*value))]).collect();
                    stack.remove(0);
                }else{
                    return MethodResult::MachineError("Tried to execute istore without int on top of stack");
                }
            },
            Instruction::LStore(at) => {
                if let Some(JValue::Long(value)) = stack.get(0){
                    let at = *at as usize;
                    locals = locals.splice(at..at+2, [Some(JValue::Long(*value)), Some(JValue::Second)]).collect();
                    stack.remove(0); stack.remove(0); // get rid of the Second too
                }else{
                    return MethodResult::MachineError("Tried to execute lstore without long on top of stack");
                }
            },
            Instruction::FStore(at) => {
                if let Some(JValue::Float(value)) = stack.get(0){
                    let at = *at as usize; // yeah
                    locals = locals.splice(at..at+1, [Some(JValue::Float(*value))]).collect();
                    stack.remove(0);
                }else{
                    return MethodResult::MachineError("Tried to execute fstore without float on top of stack");
                }
            },
            Instruction::DStore(at) => {
                if let Some(JValue::Double(value)) = stack.get(0){
                    let at = *at as usize;
                    locals = locals.splice(at..at+2, [Some(JValue::Double(*value)), Some(JValue::Second)]).collect();
                    stack.remove(0); stack.remove(0); // get rid of the Second too
                }else{
                    return MethodResult::MachineError("Tried to execute dstore without double on top of stack");
                }
            },
            Instruction::AStore(at) => {
                if let Some(JValue::Reference(value)) = stack.get(0){
                    let at = *at as usize;
                    locals = locals.splice(at..at+1, [Some(JValue::Reference(*value))]).collect();
                    stack.remove(0);
                }else{
                    return MethodResult::MachineError("Tried to execute astore without reference on top of stack");
                }
            },

            Instruction::ILoad(at) => {
                if let Some(Some(JValue::Int(value))) = locals.get(*at as usize){
                    stack.insert(0, JValue::Int(*value));
                }else{
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
            Instruction::ALoad(at) => {
                if let Some(Some(JValue::Reference(value))) = locals.get(*at as usize){
                    stack.insert(0, JValue::Reference(*value));
                }else{
                    return MethodResult::MachineError("Tried to execute aload without reference at local variable index");
                }
            },

            Instruction::Goto(offset) => {
                let target = (*idx as isize) + (*offset as isize);
                if target < 0{
                    panic!("Bad goto offset");
                }
                i = bytecode_idx_to_instr_idx(target as usize, code);
                was_jump = true;
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
            Instruction::Return => return MethodResult::Finish,

            Instruction::GetField(target) => {
                let owner = heap::get_or_create_bt_class(format!("L{};", target.owner_name.clone()))
                    .expect("Could not load field owner")
                    .ensure_loaded()
                    .expect("Could not load field owner");
                let mut pushed = false;
                for f in &owner.static_fields{
                    if f.0.name == target.name_and_type.name{
                        let j_value = f.1.clone();
                        println!("got {:?} from field {:?}", j_value, &target.name_and_type);
                        stack.insert(0, j_value);
                        pushed = true;
                    }
                }
                if !pushed{
                    todo!();
                }
            }
            
            Instruction::InvokeVirtual(target) => {
                let receiver = stack.pop();
                if let Some(JValue::Reference(Some(r))) = receiver{
                    let class = &r.deref().class;
                    let method = class.virtual_method(&target.name_and_type)
                        .expect("Tried to execute invokevirtual for method that doesn't exist on receiver");
                    let num_params = method.parameters.len();
                    let mut args = Vec::with_capacity(num_params + 1);
                    args.push(receiver.unwrap().clone());
                    for _ in 0..num_params{
                        args.push(stack.pop().expect("Tried to execute invokevirtual with insufficient arguments"));
                    }
                    let result = execute(&method, args);
                    // TODO: exception handling
                    match result{
                        MethodResult::FinishWithValue(v) => stack.push(v),
                        MethodResult::Finish => {},
                        MethodResult::Throw(e) => return MethodResult::Throw(e),
                        MethodResult::MachineError(e) => return MethodResult::MachineError(e),
                    }
                }else if let Some(JValue::Reference(None)) = receiver{
                    return MethodResult::Throw(JValue::Reference(None)); // TODO: synthesize NPEs
                }else{
                    return MethodResult::MachineError("Tried to execute invokevirtual without object on stack");
                }
            }

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