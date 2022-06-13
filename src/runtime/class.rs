use std::sync::Arc;
use crate::parser::{classfile_structs::{Code, Classfile, NameAndType}, classfile_parser};
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

    pub fn virtual_method(&self, target: &NameAndType) -> Option<&Method>{
        for method in &self.methods{
            if method.name == target.name && method.descriptor() == target.descriptor {
                return Some(method);
            }
        }
        if let Some(c) = &self.super_class{
            return c.virtual_method(target);
        }
        return None;
    }

    pub fn interface_method(&self, target: &NameAndType) -> Option<&Method>{
        for method in &self.methods{
            if method.name == target.name && method.descriptor() == target.descriptor {
                return Some(method);
            }
        }
        if let Some(c) = &self.super_class{
            if let Some(m) = c.interface_method(target){
                return Some(m);
            }
        }
        for interface in &self.interfaces{
            if let Some(m) = interface.interface_method(target){
                return Some(m);
            }
        }
        return None;
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
    pub name: String,
    pub parameters: Vec<ClassRef>,
    pub return_type: ClassRef,
    pub visibility: Visibility,
    pub code: MethodImpl
}

impl Method {
    pub fn descriptor(&self) -> String{
        let mut desc = String::with_capacity(self.parameters.len() + 2);
        desc.push_str("(");
        for param in &self.parameters{
            desc.push_str(&param.descriptor);
        }
        desc.push_str(")");
        desc.push_str(&self.return_type.descriptor);
        return desc;
    }
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

