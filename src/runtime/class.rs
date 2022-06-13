use std::sync::Arc;
use crate::parser::{classfile_structs::{Code, Classfile}, classfile_parser};
use super::{classes::{ClassLoader, self}, jvalue::JValue};

#[derive(Debug)]
pub struct Class{
    pub name: String,           // a.b.C
    pub descriptor: String,     // La/b/C; or I or [I...
    pub super_class: Option<ClassRef>, // None for Object and primitives
    pub interfaces: Vec<ClassRef>,
    pub loader_name: String,
    pub instance_fields: Vec<Field>,
    pub static_fields: Vec<(Field, JValue)>,
    pub methods: Vec<Method>
}

impl Class{
    pub fn assignable_to(&self, to: &ClassRef) -> bool{
        if self.descriptor == to.descriptor{
            return true;
        }
        if let Some(sup) = &self.super_class && sup.assignable_to(to){
            return true;
        }
        for interface in &self.interfaces{
            if interface.assignable_to(to){
                return true;
            }
        }
        return false;
    }
}

pub type ClassRef = Arc<Class>;

#[derive(Debug)]
pub struct Field{
    name: String,
    type_class: ClassRef, // TODO: does a field of the same type as the class create cycles?
    visibility: Visibility
}

#[derive(Debug)]
pub struct Method{
    name: String,
    parameters: Vec<ClassRef>,
    return_type: ClassRef,
    visibility: Visibility,
    code: MethodImpl
}

#[derive(Debug)]
pub enum Visibility{
    Public, Local, Protected, Private
}

#[derive(Debug)]
pub enum MethodImpl{
    Bytecode(Code), Native, Abstract
}




// Class loading
// TODO: detect circular hierarchies

// Loads and links the class with the given name using the bootstrap classloader.
pub fn load_class(classname: String) -> Result<Class, &'static str>{
    return load_class_with(classname, Arc::new(classes::BOOTSTRAP_LOADER));
}

/// Loads and links the class with the given name, provided by the given classloader.
pub fn load_class_with(classname: String, loader: Arc<dyn ClassLoader>) -> Result<Class, &'static str>{
    return classfile_to_class(classfile_parser::parse(&mut loader.load(&classname))?, loader);
}

// Links the classfile into a class, ascribing it to the given classloader.
pub fn classfile_to_class(classfile: Classfile, loader: Arc<dyn ClassLoader>) -> Result<Class, &'static str>{
    return Ok(Class{
        name: binary_to_fq_name(classfile.name.clone()),
        descriptor: format!("L{};", classfile.name.clone()),
        loader_name: loader.name(),
        instance_fields: vec![],
        static_fields: vec![],
        methods: vec![],
        super_class: None,
        interfaces: vec![],
    });
}

fn binary_to_fq_name(binary_name: String) -> String{
    return binary_name.replace("/", ".");
}

