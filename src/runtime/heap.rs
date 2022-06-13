use std::{sync::{RwLock, Arc}, collections::HashMap};

use crate::constants;

use super::{jvalue::JObject, class::ClassRef};

// Heap shared between threads.

// References are indexes into the "active" list.
// On GC, reachable objects are moved to the "inactive", and the two lists are swapped.

#[derive(Debug, Clone, Copy)]
pub struct JRef{
    heap_idx: usize // used in `get`
}

// Heaps must be mutable so that they can be setup at runtime
// RwLocks are used for threadsafe addition to heaps and classloading
static mut HEAP_ACTIVE: Option<RwLock<Vec<Arc<JObject>>>> = None;
static mut HEAP_INACTIVE: Option<RwLock<Vec<Arc<JObject>>>> = None;
// Map of classloader name -> class name -> class
static mut LOADED_CLASSES: Option<RwLock<HashMap<String, HashMap<String, ClassRef>>>> = None;

pub fn setup(){
    unsafe{
        HEAP_ACTIVE = Some(RwLock::new(Vec::new()));
        HEAP_INACTIVE = Some(RwLock::new(Vec::new()));
        LOADED_CLASSES = Some(RwLock::new(HashMap::new()));
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

pub fn classes_by_loader(loader_name: String) -> Vec<ClassRef>{
    unsafe{
        let rw = LOADED_CLASSES.as_ref().unwrap();
        let loaded_classes = &*rw.read().unwrap();
        let classes_by_loader = loaded_classes.get(&loader_name);
        match classes_by_loader{
            Some(classes) => classes.values().cloned().collect(),
            None => Vec::new()
        }
    }
}

pub fn class_by_name(loader_name: String, classname: String) -> Option<ClassRef>{
    for class in classes_by_loader(loader_name){
        if class.name == classname{
            return Some(class.clone());
        }
    }
    return None;
}

pub fn bt_class_by_name(name: String) -> Option<ClassRef>{
    return class_by_name(constants::BOOTSTRAP_LOADER_NAME.to_owned(), name);
}