#![feature(absolute_path)]
#![feature(let_else)]
#![feature(if_let_guard)]
#![feature(let_chains)]

mod constants;
mod runtime;
mod parser;

fn main() {
    // let's parse a classfile to start with
    // very, very painfully and slowly :)
    // let mut buf = String::new();
    // io::stdin().read_line(&mut buf).expect("Need to specify a classfile");
    // let buf = buf.trim_end();

    // let path = path::Path::new(&buf);
    // let path = path::absolute(path).expect("Could not make path absolute");
    // let mut classfile = File::open(path).expect("Expected a classfile to exist!");

    // let mut class: Vec<u8> = Vec::new();
    // classfile.read_to_end(&mut class).expect("Unable to read classfile data");

    // let result = parser::classfile_parser::parse(&mut class);

    // match result{
    //     Ok(ref u) => println!("{:?}", u),
    //     Err(u) => panic!("oh no: {}", u)
    // }

    // runtime::heap::setup();

    // // very temporary ofc
    // let mut method = String::new();
    // io::stdin().read_line(&mut method).expect("Need to specify a method");
    // let method = method.trim_end();
    // // find the method with the given name and run it
    // let o = result.expect("").methods;
    // for m in o {
    //     if &m.name == method{
    //         // find code attribute
    //         for attribute in &m.attributes {
    //             if let Code(c) = attribute {
    //                 let u = runtime::interpreter::interpret(&m, vec![JValue::Int(1)], &c);
    //                 match u{
    //                     Ok(u) => println!("{:?}", u),
    //                     Err(u) => panic!("oh no: {}", u)
    //                 }
    //             }
    //         }
    //     }
    // }

    runtime::heap::setup();

    match runtime::class::load_class("java/lang/String".to_owned()){
        Ok(o) => println!("{:?}", o),
        Err(e) => println!("error: {}", e),
    }
}
