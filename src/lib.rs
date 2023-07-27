#![feature(thread_local)]
#![feature(ptr_metadata)]

mod emptyable_mutex;
mod inlinable_ptr;

use std::sync::Arc;
use std::sync::atomic::AtomicPtr;
use crate::emptyable_mutex::Mutex;

static GLOBAL_TASK_QUEUE: Mutex<Vec<Arc<AtomicPtr<TaskNode>>>> = Mutex::new_empty(Vec::new());
#[thread_local]
static LOCAL_TASK_QUEUE: Option<Arc<AtomicPtr<TaskNode>>> = None;

struct TaskNode {
    callback: Box<dyn FnOnce()>,
}

struct InlinedPtr<T: ?Sized> {
    call_ptr: fn() -> CallResult,

}

pub enum CallResult {
    Finished,
    Wait,
}
