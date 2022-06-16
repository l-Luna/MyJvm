use std::{sync::{RwLock, Arc}, collections::HashMap, hash::Hash};

use crate::{constants, parser::classfile_structs::Classfile};

use super::{jvalue::JObject, class::{ClassRef, Class, MaybeClass}, classes::{self, ClassLoader}};

// TODO: use weak references everywhere (esp JRef and ClassRef)
// and only keep objects and classes alive via the heaps
// since those are GC'd and cycles naturally form goddamn everywhere

// Heap shared between threads.

// References are indexes into the "active" list.
// On GC, reachable objects are moved to the "inactive", and the two lists are swapped.

#[derive(Debug, Clone, Copy)]
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

pub fn gc(){
    // Starting from GC roots, find all objects in "active" and move to "inactive",
    // Then swap and clear.
    // TODO: how do we update JRefs to match?
    //   should I just use Arc directly? how to compare reachability that way?
}

// Class handling
// TODO: move to classes.rs?

/// Adds a loaded class under the given classloader.
pub fn add_class(class: Class, loader_name: String){
    unsafe{ add_to_map_list(loader_name, Arc::new(class), &LOADED_CLASSES); }
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

/// Returns the class with the given name loaded by the given classloader.
pub fn class_by_name(loader_name: String, classname: String) -> Option<ClassRef>{
    for class in classes_by_loader(loader_name){
        if class.name == classname{
            return Some(class.clone());
        }
    }
    return None;
}

/// Adds a class under the bootstrap classloader.
pub fn add_bt_class(class: Class){
    add_class(class, constants::BOOTSTRAP_LOADER_NAME.to_owned());
}

// Returns the class with the given name loaded by the bootstrap classloader.
pub fn bt_class_by_name(name: String) -> Option<ClassRef>{
    return class_by_name(constants::BOOTSTRAP_LOADER_NAME.to_owned(), name);
}

pub fn get_or_create_class(name: String, loader: Arc<dyn ClassLoader>) -> MaybeClass{
    return match class_by_name(loader.name(), name.clone()){
        Some(r) => MaybeClass::Class(r),
        None => {
            
            MaybeClass::Unloaded(name)
        }
    };
}

// implementation

fn add_to_map_list<K, V>(key: K, value: V, map_list: &Option<RwLock<HashMap<K, Vec<V>>>>) where K: Eq + Clone + Hash{
    let rw = map_list.as_ref().unwrap();
    let loaded_classes = &mut *rw.write().unwrap();
    let loader_classes = if loaded_classes.contains_key(&key){
        loaded_classes.get_mut(&key).unwrap()
    }else{
        loaded_classes.insert(key.clone(), Vec::with_capacity(1));
        loaded_classes.get_mut(&key).unwrap()
    };
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