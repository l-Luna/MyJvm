use std::{sync::{RwLock, Arc}, collections::HashMap, hash::Hash};
use std::fmt::Debug;
use runtime::interpreter;
use runtime::jvalue::JValue;

use crate::{constants, parser::{classfile_structs::Classfile, classfile_parser}};

use super::{jvalue::JObject, class::{ClassRef, Class, MaybeClass, self}, classes::{self, ClassLoader}};

// TODO: use weak references everywhere (esp JRef and ClassRef)
// and only keep objects and classes alive via the heaps
// since those are GC'd and cycles naturally form goddamn everywhere

// Heap shared between threads.

// References are indexes into the "active" list.
// On GC, reachable objects are moved to the "inactive", and the two lists are swapped.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JRef{
    heap_idx: usize // used in `get`
}

impl JRef {
    pub fn deref(&self) -> Arc<JObject>{
        return get(self);
    }
}

// Heaps must be mutable so that they can be setup at runtime
// RwLocks are used for threadsafe addition to heaps and classloading
static mut HEAP_ACTIVE: Option<RwLock<Vec<Arc<JObject>>>> = None;
static mut HEAP_INACTIVE: Option<RwLock<Vec<Arc<JObject>>>> = None;
// Map of classloader name -> associated classes
static mut CREATED_CLASSES: Option<RwLock<HashMap<String, Vec<Classfile>>>> = None;
static mut LOADED_CLASSES: Option<RwLock<HashMap<String, Vec<ClassRef>>>> = None;

pub fn setup(){
    unsafe{
        HEAP_ACTIVE = Some(RwLock::new(Vec::new()));
        HEAP_INACTIVE = Some(RwLock::new(Vec::new()));
        CREATED_CLASSES = Some(RwLock::new(HashMap::new()));
        LOADED_CLASSES = Some(RwLock::new(HashMap::new()));
    }

    for primitive in classes::create_primitive_classes(){
        add_bt_class(primitive);
    }
    classes::setup_java_base();
}

// Object handling

pub fn add(obj: JObject) -> JRef{
    unsafe{
        let rw = HEAP_ACTIVE.as_ref().unwrap();
        let true_heap = &mut *rw.write().unwrap();
        true_heap.push(Arc::new(obj));
        return JRef{ heap_idx: true_heap.len() };
    }
}

pub fn get(refs: &JRef) -> Arc<JObject>{
    unsafe{
        let rw = HEAP_ACTIVE.as_ref().unwrap();
        let true_heap = rw.read().unwrap();
        return true_heap[refs.heap_idx].clone();
    }
}

pub fn add_ref(obj: JObject) -> JValue{
    return JValue::Reference(Some(add(obj)));
}

pub fn gc(){
    // Starting from GC roots, find all objects in "active" and move to "inactive",
    // Then swap and clear.
    // TODO: how do we update JRefs to match?
    //   should I just use Arc directly? how to compare reachability that way?
}

// Class handling
// TODO: move to classes.rs?

/// Adds a loaded class under the given classloader, and invokes its static initializer.
pub fn add_class(class: Class, loader_name: String){
    let class_desc = class.descriptor.clone();
    unsafe{ add_to_map_list(loader_name.clone(), Arc::new(class), &LOADED_CLASSES); }
    // TODO: don't repeat this (get().unwrap().ensure().unwrap()) as much
    let class = get_or_create_bt_class(class_desc).unwrap().ensure_loaded().unwrap();
    if let Some(clinit) = class.static_method(&constants::clinit()){
        interpreter::execute(clinit, Vec::new());
    }
}

/// Returns a "snapshot" of the classes loaded by the given loader.
pub fn classes_by_loader(loader_name: String) -> Vec<ClassRef>{
    unsafe{ return unwrap_map_list(loader_name, &LOADED_CLASSES); }
}

/// Adds a created classfile under the given classloader to be used in further loading or linking.
pub fn add_classfile(class: Classfile, loader_name: String){
    unsafe{ add_to_map_list(loader_name, class, &CREATED_CLASSES); }
}

/// Returns a "snapshot" of the classesfiles created by the given loader.
pub fn classfiles_by_loader(loader_name: String) -> Vec<Classfile>{
    unsafe{ return unwrap_map_list(loader_name, &CREATED_CLASSES); }
}

/// Returns the class with the given descriptor loaded by the given classloader.
pub fn class_by_desc(loader_name: String, class_desc: String) -> Option<ClassRef>{
    for class in classes_by_loader(loader_name){
        if class.descriptor == class_desc{
            return Some(class.clone());
        }
    }
    return None;
}

/// Returns the classfile with the given name loaded by the given classloader.
pub fn classfile_by_name(loader_name: String, class_desc: String) -> Option<Classfile>{
    for class in classfiles_by_loader(loader_name){
        if class.name == class_desc{
            return Some(class.clone());
        }
    }
    return None;
}

/// Adds a class under the bootstrap classloader.
pub fn add_bt_class(class: Class){
    add_class(class, constants::BOOTSTRAP_LOADER_NAME.to_owned());
}

// Returns the class with the given descriptor loaded by the bootstrap classloader.
pub fn bt_class_by_desc(class_desc: String) -> Option<ClassRef>{
    return class_by_desc(constants::BOOTSTRAP_LOADER_NAME.to_owned(), class_desc);
}

pub fn get_or_create_class(class_desc: String, loader: &Arc<dyn ClassLoader>) -> Result<MaybeClass, String>{
    return match class_by_desc(loader.name(), class_desc.clone()){
        Some(r) => Ok(MaybeClass::Class(r)),
        None => {
            if class_desc.starts_with("["){
                let name = class_desc[1..].to_owned();
                return Ok(MaybeClass::UnloadedArray(name));
            }
            // TODO: also check primitive classes?
            if let Some(_) = classfile_by_name(loader.name(), desc_to_name(class_desc.clone())?){
                Ok(MaybeClass::Unloaded(class_desc))
            }else{
                let mut data = loader.load(&desc_to_name(class_desc.clone())?);
                let classfile = classfile_parser::parse(&mut data)?;
                add_classfile(classfile, loader.name());
                Ok(MaybeClass::Unloaded(class_desc))
            }
        }
    };
}

pub fn get_or_create_bt_class(class_desc: String) -> Result<MaybeClass, String>{
    let u: Arc<dyn ClassLoader> = Arc::new(classes::BOOTSTRAP_LOADER);
    return get_or_create_class(class_desc, &u);
}

// TODO: handle user classloaders
pub fn ensure_loaded(class: &MaybeClass) -> Result<ClassRef, String>{
    match class{
        MaybeClass::Class(c) => Ok(c.clone()),
        MaybeClass::Unloaded(desc) => {
            // first check if the class has already been loaded
            if let Some(c) = bt_class_by_desc(desc.clone()){
                return Ok(c);
            }
            // otherwise create and save it
            let class = class::load_class(desc_to_name(desc.clone())?)?;
            add_bt_class(class);
            return Ok(bt_class_by_desc(desc.clone()).unwrap());
        },
        // TODO: cache array classes?
        MaybeClass::UnloadedArray(comp_desc) => Ok(Arc::new(
            classes::array_class(&ensure_loaded(&get_or_create_bt_class(comp_desc.clone())?)?)
        )),
    }
}

// implementation

fn desc_to_name(desc: String) -> Result<String, String>{
    return if desc.starts_with("L") && desc.ends_with(";"){
        Ok(desc[1..desc.len() - 1].to_string())
    }else if desc.starts_with("["){
        Ok(desc_to_name(desc[1..].to_owned())? + "[]")
    }else{
        // sure
        match desc.chars().nth(0){
            None => Err("Invalid descriptor of length 0".to_owned()),
            Some('Z') => Ok("boolean".to_owned()),
            Some('B') => Ok("byte".to_owned()),
            Some('S') => Ok("short".to_owned()),
            Some('C') => Ok("char".to_owned()),
            Some('I') => Ok("int".to_owned()),
            Some('J') => Ok("long".to_owned()),
            Some('F') => Ok("float".to_owned()),
            Some('D') => Ok("double".to_owned()),
            Some('V') => Ok("void".to_owned()),
            Some(c) => Err(format!("Invalid descriptor character: {}", c))
        }
    }
}

fn add_to_map_list<K, V>(key: K, value: V, map_list: &Option<RwLock<HashMap<K, Vec<V>>>>) where K: Eq + Clone + Hash, V: PartialEq + Debug{
    let rw = map_list.as_ref().unwrap();
    let loaded_classes = &mut *rw.write().unwrap();
    let loader_classes = if loaded_classes.contains_key(&key){
        loaded_classes.get_mut(&key).unwrap()
    }else{
        loaded_classes.insert(key.clone(), Vec::with_capacity(1));
        loaded_classes.get_mut(&key).unwrap()
    };
    if loader_classes.contains(&value){
        panic!("Entry already exists: {:?}", &value);
    }
    loader_classes.push(value);
}

pub fn unwrap_map_list<K, V>(key: K, map_list: &Option<RwLock<HashMap<K, Vec<V>>>>) -> Vec<V> where K: Eq + Clone + Hash, V: Clone{
    let rw = map_list.as_ref().unwrap();
    let loaded_classes = rw.read().unwrap();
    let classes_by_loader = loaded_classes.get(&key);
    match classes_by_loader{
        Some(classes) => classes.clone(),
        None => Vec::new()
    }
}