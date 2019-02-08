use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use core::alloc::{GlobalAlloc, Layout};
use core::cmp::Ordering;
use core::mem;
use spin::Mutex;

use context::arch;
use context::memory::{Memory, SharedMemory};
/// Unique identifier for a context (i.e. `pid`).
use ::core::sync::atomic::AtomicUsize;
int_like!(ContextId, AtomicContextId, usize, AtomicUsize);

/// The status of a context - used for scheduling
/// See `syscall::process::waitpid` and the `sync` module for examples of usage
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Runnable,
    Blocked,
    Stopped(usize),
    Exited(usize),
}

/// A context, which identifies either a process or a thread
#[derive(Debug)]
pub struct Context {
    /// The ID of this context
    pub id: ContextId,
    /// The group ID of this context
    pub pgid: ContextId,
    /// The ID of the parent context
    pub ppid: ContextId,
    /// Status of context
    pub status: Status,
    /// Context running or not
    pub running: bool,
    /// CPU ID, if locked
    pub cpu_id: Option<usize>,
    /// The architecture specific context
    pub arch: arch::Context,
    /// Kernel FX - used to store SIMD and FPU registers on context switch
    pub kfx: Option<Box<[u8]>>,
    /// Kernel stack
    pub kstack: Option<Box<[u8]>>,
    /// User heap
    pub heap: Option<SharedMemory>,
    /// User stack
    pub stack: Option<Memory>,
    /// The name of the context
    pub name: Arc<Mutex<Box<[u8]>>>,
}

