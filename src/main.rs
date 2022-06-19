#![feature(absolute_path)]
#![feature(let_else)]
#![feature(if_let_guard)]
#![feature(let_chains)]

mod constants;
mod runtime;
mod parser;

fn main() {
    runtime::heap::setup();

    match runtime::class::load_class("run/Basics".to_owned()){
        Ok(o) => {
            println!("{:?}", o);
            for m in o.methods{
                if m.name == "main2"{
                    let result = runtime::interpreter::execute(&m, vec![]);
                    println!("got {:?}", result);
                }
            }
        },
        Err(e) => println!("error: {}", e),
    }
}
