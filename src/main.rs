#![feature(if_let_guard)]
#![feature(let_chains)]

use crate::runtime::interpreter::StackTrace;

mod constants;
mod runtime;
mod parser;

fn main() {
    runtime::heap::setup();

    match runtime::class::load_class("run/Basics".to_owned()){
        Ok(o) => {
            for m in &o.methods{
                if m.name == "main2"{
                    let result = runtime::interpreter::execute(&o, &m, vec![], StackTrace::new());
                    println!("got {:?}", result);
                }
            }
        },
        Err(e) => println!("error: {}", e),
    }
}
