use crate::constants;

use super::{class::ClassRef, heap::{JRef, self}};

// Defines class loading and the bootstrap classloader

pub trait ClassLoader{
    fn name(&self) -> String;
    fn load(&self, classname: &str) -> Vec<u8>;
    fn prev_loaded(&self) -> Vec<ClassRef>{
        return heap::classes_by_loader(self.name());
    }
}

const BOOTSTRAP_LOADER: BootstrapLoader = BootstrapLoader{};

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

struct BootstrapLoader{}

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