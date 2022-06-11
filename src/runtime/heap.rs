use std::{sync::{RwLock, Arc}, ops::Deref};

use super::jvalue::JObject;

// Heap shared between threads.

// References are indexes into the "active" list.
// On GC, reachable objects are moved to the "inactive", and the two lists are swapped.

#[derive(Debug, Clone, Copy)]
pub struct JRef{
    heap_idx: usize // used in `get`
}

// Heaps must be mutable so that they can be setup at runtime
// RwLocks are used for threadsafe addition to heaps
static mut HEAP_ACTIVE: Option<RwLock<Vec<Arc<JObject>>>> = None;
static mut HEAP_INACTIVE: Option<RwLock<Vec<Arc<JObject>>>> = None;

pub fn setup(){
    unsafe{
        HEAP_ACTIVE = Some(RwLock::new(Vec::new()));
        HEAP_INACTIVE = Some(RwLock::new(Vec::new()));
    }
}

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
    // Then swap and clear
}