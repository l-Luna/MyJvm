use std::sync::{Arc, RwLock};
use crate::{parser::{classfile_structs::{Code, Classfile, NameAndType, FieldInfo, MethodInfo, Attribute, LineNumberMapping}, classfile_parser}, constants};
use super::{classes::{ClassLoader, self}, jvalue::JValue, heap};

#[derive(Debug)]
pub struct Class{
    pub name: String,           // a.b.C
    pub descriptor: String,     // La/b/C; or I or [I...
    pub super_class: Option<ClassRef>, // None for Object and primitives
    pub interfaces: Vec<ClassRef>,
    pub loader_name: String,
    pub instance_fields: Vec<Field>,
    pub static_fields: Vec<RwLock<(Field, JValue)>>,
    pub methods: Vec<Method>
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool{
        return self.name == other.name
            && self.descriptor == other.descriptor;
    }
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

    pub fn static_method(&self, target: &NameAndType) -> Option<&Method>{
        for method in &self.methods{
            if method.is_static && method.name == target.name && method.descriptor() == target.descriptor{
                return Some(method);
            }
        }
        return None;
    }

    pub fn special_method(&self, target: &NameAndType, owner_int_name: String) -> Option<(&Method, &Class)>{
        if self.descriptor == format!("L{};", owner_int_name){
            for method in &self.methods{
                if !method.is_static && method.name == target.name && method.descriptor() == target.descriptor{
                    return Some((method, self));
                }
            }
        }
        if let Some(c) = &self.super_class{
            if let Some(ret) = c.special_method(target, owner_int_name.clone()){
                return Some(ret);
            }
        }
        for interface in &self.interfaces{
            if let Some(m) = interface.special_method(target, owner_int_name.clone()){
                return Some(m);
            }
        }
        return None;
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

#[derive(Debug, Clone, PartialEq)]
pub enum MaybeClass{
    Class(ClassRef),
    Unloaded(String), // TODO: privatise ctor? need to ensure classfile is created first
    UnloadedArray(String)
}

#[derive(Debug, PartialEq)]
pub struct Field{
    pub name: String,
    pub type_class: MaybeClass, // TODO: does a field of the same type as the class create cycles?
    pub visibility: Visibility,
    pub is_static: bool
}

#[derive(Debug, PartialEq)]
pub struct Method{
    pub name: String,
    pub parameters: Vec<MaybeClass>,
    pub return_type: MaybeClass,
    pub visibility: Visibility,
    pub is_static: bool,
    pub line_number_table: Option<Vec<LineNumberMapping>>,
    pub code: MethodImpl
}

impl MaybeClass{
    pub fn descriptor(&self) -> String{
        return match self{
            MaybeClass::Unloaded(d) => d.clone(),
            MaybeClass::Class(c) => c.descriptor.clone(),
            MaybeClass::UnloadedArray(of) => "[".to_owned() + &of.clone()
        };
    }

    pub fn ensure_loaded(&self) -> Result<ClassRef, String> {
        return heap::ensure_loaded(&self);
    }
}

impl Method{
    pub fn descriptor(&self) -> String{
        let mut desc = String::with_capacity(self.parameters.len() + 2);
        desc.push_str("(");
        for param in &self.parameters{
            desc.push_str(&param.descriptor());
        }
        desc.push_str(")");
        desc.push_str(&self.return_type.descriptor());
        return desc;
    }
}

#[derive(Debug, PartialEq)]
pub enum Visibility{
    Public, Local, Protected, Private
}

#[derive(Debug, PartialEq)]
pub enum MethodImpl{
    Bytecode(Code), Native, Abstract
}




// Class loading
// TODO: detect circular hierarchies

/// Loads and links the class with the given name using the bootstrap classloader.
pub fn load_class(classname: String) -> Result<Class, String>{
    return load_class_with(classname, Arc::new(classes::BOOTSTRAP_LOADER));
}

/// Loads and links the class with the given name, provided by the given classloader.
pub fn load_class_with(classname: String, loader: Arc<dyn ClassLoader>) -> Result<Class, String>{
    return link_class(classfile_parser::parse(&mut loader.load(&classname))?, loader);
}

/// Links the classfile into a class, ascribing it to the given classloader.
pub fn link_class(classfile: Classfile, loader: Arc<dyn ClassLoader>) -> Result<Class, String>{
    let all_fields: Vec<_> = classfile.fields.into_iter()
        .map(|f| link_field(f, &loader))
        .collect();
    let mut instance_fields = Vec::new();
    let mut static_fields = Vec::new();
    for f in all_fields{
        let f = f?;
        if f.is_static{
            static_fields.push(f);
        }else{
            instance_fields.push(f);
        }
    }
    let static_fields = static_fields.into_iter()
        .map(|f| (JValue::default_value_for(&f.type_class.descriptor()), f))
        .map(|(a, b)| (b, a)) // :3
        .map(RwLock::new)
        .collect();

    let mut all_methods = Vec::with_capacity(classfile.methods.len());
    for m in classfile.methods{
        all_methods.push(link_method(m, &loader)?);
    }

    let mut super_class = None;
    if let Some(super_name) = &classfile.super_class{
        super_class = Some(heap::get_or_create_class(format!("L{};", super_name), &loader)
            .expect("Could not find or parse super-class classfile")
            .ensure_loaded()
            .expect("Could not link superclass"));
    }

    return Ok(Class{
        name: binary_to_fq_name(classfile.name.clone()),
        descriptor: format!("L{};", classfile.name.clone()),
        loader_name: loader.name(),
        instance_fields,
        static_fields,
        methods: all_methods,
        super_class,
        interfaces: vec![],
    });
}

fn binary_to_fq_name(binary_name: String) -> String{
    return binary_name.replace("/", ".");
}

fn flags_to_visibility(flags: u16) -> Visibility{
    return if constants::bit_set(flags, constants::ACC_PUBLIC){
        Visibility::Public
    }else if constants::bit_set(flags, constants::ACC_PROTECTED){
        Visibility::Public
    }else if constants::bit_set(flags, constants::ACC_PRIVATE){
        Visibility::Private
    }else{
        Visibility::Local
    }
}

fn link_field(field: FieldInfo, loader: &Arc<dyn ClassLoader>) -> Result<Field, String>{
    return Ok(Field{
        name: field.name,
        type_class: heap::get_or_create_class(field.desc, loader)?,
        visibility: flags_to_visibility(field.flags),
        is_static: constants::bit_set(field.flags, constants::ACC_STATIC)
    });
}

fn link_method(method: MethodInfo, loader: &Arc<dyn ClassLoader>) -> Result<Method, String>{
    let mut desc = method.desc.clone();
    let return_type = desc.remove(method.desc.len() - 1);
    let return_type = heap::get_or_create_class(return_type, loader)?;
    let mut parameters = Vec::with_capacity(desc.len());
    for d in desc{
        parameters.push(heap::get_or_create_class(d, loader)?);
    }

    // TODO: check Code presence & flags
    // (should be checked much earlier though)
    let mut code = MethodImpl::Abstract;
    let mut line_number_table = None;
    for attr in method.attributes{
        if let Attribute::Code(c) = attr{
            code = MethodImpl::Bytecode(c);
        }else if let Attribute::LineNumberTable(table) = attr{
            line_number_table = Some(table);
        }
    }
    if constants::bit_set(method.flags, constants::METHOD_ACC_NATIVE){
        code = MethodImpl::Native;
    }

    return Ok(Method{
        name: method.name,
        parameters,
        return_type,
        visibility: flags_to_visibility(method.flags),
        is_static: constants::bit_set(method.flags, constants::ACC_STATIC),
        line_number_table,
        code,
    });
}