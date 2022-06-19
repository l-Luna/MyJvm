use std::sync::Arc;
use crate::{parser::{classfile_structs::{Code, Classfile, NameAndType, FieldInfo, MethodInfo, Attribute}, classfile_parser}, constants};
use super::{classes::{ClassLoader, self}, jvalue::JValue, heap};

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
pub enum MaybeClass{
    Class(ClassRef),
    Unloaded(String), // TODO: privatise ctor? need to ensure classfile is created first
    UnloadedArray(String)
}

#[derive(Debug)]
pub struct Field{
    pub name: String,
    pub type_class: MaybeClass, // TODO: does a field of the same type as the class create cycles?
    pub visibility: Visibility,
    pub is_static: bool
}

#[derive(Debug)]
pub struct Method{
    pub name: String,
    pub parameters: Vec<MaybeClass>,
    pub return_type: MaybeClass,
    pub visibility: Visibility,
    pub is_static: bool,
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
        .collect();
    let mut all_methods = Vec::with_capacity(classfile.methods.len());
    for m in classfile.methods{
        all_methods.push(link_method(m, &loader)?);
    }
    return Ok(Class{
        name: binary_to_fq_name(classfile.name.clone()),
        descriptor: format!("L{};", classfile.name.clone()),
        loader_name: loader.name(),
        instance_fields,
        static_fields,
        methods: all_methods,
        super_class: None,
        interfaces: vec![],
    });
}

fn binary_to_fq_name(binary_name: String) -> String{
    return binary_name.replace("/", ".");
}

fn flags_to_visibility(flags: u16) -> Visibility{
    if constants::bit_set(flags, constants::ACC_PUBLIC){
        return Visibility::Public;
    }else if constants::bit_set(flags, constants::ACC_PROTECTED){
        return Visibility::Public;
    }else if constants::bit_set(flags, constants::ACC_PRIVATE){
        return Visibility::Private;
    }else{
        return Visibility::Local;
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
    let mut code: MethodImpl = MethodImpl::Abstract;
    for attr in method.attributes{
        if let Attribute::Code(c) = attr {
            code = MethodImpl::Bytecode(c);
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
        code,
    });
}