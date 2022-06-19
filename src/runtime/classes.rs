use std::{path, fs, sync::RwLock, collections::HashMap, io::Read};
use std::sync::Arc;

use crate::constants;

use super::{class::{ClassRef, Class}, heap::{JRef, self}};

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

static mut JAVA_BASE_CLASSES: Option<RwLock<HashMap<String, Vec<u8>>>> = None;
static mut ARRAY_CLASSES: Option<RwLock<HashMap<String, Vec<u8>>>> = None;

impl ClassLoader for BootstrapLoader{
    fn name(&self) -> String{
        return constants::BOOTSTRAP_LOADER_NAME.to_owned();
    }
    fn load(&self, classname: &str) -> Vec<u8> {
        unsafe{
            let rw = JAVA_BASE_CLASSES.as_ref().unwrap();
            let class_data = &mut *rw.write().unwrap();
            dbg!(classname);
            return class_data.get(classname).expect("java.base class not found!").clone();
        }
    }
}

pub fn setup_java_base(){
    // TODO: also handle user classes
    let mut java_home: Option<String> = None;
    for op in std::env::args() {
        if op.starts_with("-java_home="){
            java_home = Some(op[11..].to_owned());
        }
    }
    if java_home == None{
        java_home = Some(std::env::var("JAVA_HOME").expect("The \"JAVA_HOME\" variable must be set, or \"-java_home=...\" must be given as argument."));
    }
    let java_home = java_home.unwrap();

    // yes this isn't strictly correct I know
    let java_base = format!("{}/jmods/java.base.jmod", java_home);
    let path = path::Path::new(&java_base);
    println!("Looking for java.base at {}", java_base);
    let file = fs::File::open(&path).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let mut data = HashMap::new();
    for u in 0..zip.len(){
        let mut file = zip.by_index(u).unwrap();
        let name = file.name().to_owned();
        if name.starts_with("classes/") && name.ends_with(".class"){
            let mut file_data = Vec::new();
            file.read_to_end(&mut file_data).expect("Could not read java.base class data!");
            data.insert(name[8..name.len() - 6].to_owned(), file_data);
        }
    }
    
    unsafe{
        JAVA_BASE_CLASSES = Some(RwLock::new(data));
        ARRAY_CLASSES = Some(RwLock::new(HashMap::new()));
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
        ( "double", "D" ),
        ( "void", "V" )
    ].into_iter().map(primitive_class).collect();
}

fn primitive_class(template: (&str, &str)) -> Class{
    return Class{
        name: template.0.to_owned(),
        descriptor: template.1.to_owned(),
        loader_name: constants::BOOTSTRAP_LOADER_NAME.to_owned(),
        instance_fields: vec![],
        static_fields: vec![],
        methods: vec![],
        super_class: None,
        interfaces: vec![],
    };
}

// Array classes

fn array_class(of: &ClassRef) -> Class{
    return Class{
        name: of.name.clone() + "[]",
        descriptor: "[".to_owned() + &of.descriptor,
        loader_name: constants::BOOTSTRAP_LOADER_NAME.to_owned(),
        instance_fields: vec![],
        static_fields: vec![],
        methods: vec![],
        super_class: Some(of.clone()),
        interfaces: vec![],
    };
}