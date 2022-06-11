#![feature(absolute_path)]
#![feature(let_else)]
#![feature(if_let_guard)]
#![feature(box_syntax)]

extern crate core;

use std::fs::File;
use std::io::Read;
use std::{io, path};

mod constants;
mod runtime;
mod parser;

fn main() {
    // let's parse a classfile to start with
    // very, very painfully and slowly :)
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).expect("Need to specify a classfile");
    let buf = buf.trim_end();

    let path = path::Path::new(&buf);
    let path = path::absolute(path).expect("Could not make path absolute");
    let mut classfile = File::open(path).expect("Expected a classfile to exist!");

    let mut class: Vec<u8> = Vec::new();
    classfile.read_to_end(&mut class).expect("Unable to read classfile data");

    match parser::classfile_parser::parse(&mut class){
        Ok(u) => println!("{:?}", u),
        Err(u) => panic!("oh no: {}", u)
    }
}
