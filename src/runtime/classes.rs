use crate::constants;

use super::{class::{ClassRef, Class}, heap::{JRef, self}};

// Primitive classes



// Class loaders

pub trait ClassLoader{
    fn name(&self) -> String;
    fn load(&self, classname: &str) -> Vec<u8>;
    fn prev_loaded(&self) -> Vec<ClassRef>{
        return heap::classes_by_loader(self.name());
    }
}

pub const BOOTSTRAP_LOADER: BootstrapLoader = BootstrapLoader{};

// User classloaders

struct UserClassLoader{
    object: JRef
}
impl ClassLoader for UserClassLoader{
    fn name(&self) -> String {
        todo!()
    }
    fn load(&self, classname: &str) -> Vec<u8> {
        todo!()
    }
}

// The bootstrap loader

pub struct BootstrapLoader{}

impl ClassLoader for BootstrapLoader{
    fn name(&self) -> String{
        return constants::BOOTSTRAP_LOADER_NAME.to_owned();
    }
    fn load(&self, classname: &str) -> Vec<u8> {
        todo!()
    }
}

// Default impls

impl<'a> std::fmt::Debug for dyn ClassLoader + 'a{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        write!(f, "Classloader {}", self.name())
    }
}

// Primitive classes

pub fn create_primitive_classes() -> Vec<Class>{
    return vec![
        ( "boolean", "Z" ),
        ( "byte", "B" ),
        ( "short", "S" ),
        ( "int", "I" ),
        ( "char", "C" ),
        ( "long", "J" ),
        ( "float", "F" ),
        ( "double", "D" )
    ].into_iter().map(primitive_class).collect();
}

fn primitive_class(template: (&str, &str)) -> Class{
    return Class{
        name: template.0.to_owned(),
        descriptor: template.1.to_owned(),
        loader_name: constants::BOOTSTRAP_LOADER_NAME.to_owned(),
        instance_fields: vec![],
        static_fields: vec![],
        methods: vec![]
    };
}