// Defines class loading and the bootstrap classloader

pub trait ClassLoader{
    fn name(&self) -> String;
    fn load(&self, classname: &str) -> Vec<u8>;
}

pub struct BootstrapLoader{}
impl ClassLoader for BootstrapLoader{
    fn name(&self) -> String{
        return "BOOTSTRAP".to_owned();
    }

    fn load(&self, classname: &str) -> Vec<u8> {
        todo!()
    }
}

impl<'a> std::fmt::Debug for dyn ClassLoader + 'a{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        write!(f, "Classloader {}", self.name())
    }
}