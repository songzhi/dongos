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

#[derive(Copy, Clone, Debug)]
pub struct WaitpidKey {
    pub pid: Option<ContextId>,
    pub pgid: Option<ContextId>,
}

impl Ord for WaitpidKey {
    fn cmp(&self, other: &WaitpidKey) -> Ordering {
        // If both have pid set, compare that
        if let Some(s_pid) = self.pid {
            if let Some(o_pid) = other.pid {
                return s_pid.cmp(&o_pid);
            }
        }

        // If both have pgid set, compare that
        if let Some(s_pgid) = self.pgid {
            if let Some(o_pgid) = other.pgid {
                return s_pgid.cmp(&o_pgid);
            }
        }

        // If either has pid set, it is greater
        if self.pid.is_some() {
            return Ordering::Greater;
        }

        if other.pid.is_some() {
            return Ordering::Less;
        }

        // If either has pgid set, it is greater
        if self.pgid.is_some() {
            return Ordering::Greater;
        }

        if other.pgid.is_some() {
            return Ordering::Less;
        }

        // If all pid and pgid are None, they are equal
        Ordering::Equal
    }
}

impl PartialOrd for WaitpidKey {
    fn partial_cmp(&self, other: &WaitpidKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for WaitpidKey {
    fn eq(&self, other: &WaitpidKey) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for WaitpidKey {}

/// A context, which identifies either a process or a thread
#[derive(Debug)]
pub struct Context {
    /// The ID of this context
    pub id: ContextId,
    /// The group ID of this context
    pub pgid: ContextId,
    /// The ID of the parent context
    pub ppid: ContextId,
    /// Process umask
    pub umask: usize,
    /// Status of context
    pub status: Status,
    /// Context running or not
    pub running: bool,
    /// CPU ID, if locked
    pub cpu_id: Option<usize>,
    /// Current system call
    pub syscall: Option<(usize, usize, usize, usize, usize, usize)>,
    /// Head buffer to use when system call buffers are not page aligned
    pub syscall_head: Box<[u8]>,
    /// Tail buffer to use when system call buffers are not page aligned
    pub syscall_tail: Box<[u8]>,
    /// Context is halting parent
    pub vfork: bool,
    /// Context is being waited on
    pub waitpid: Arc<WaitMap<WaitpidKey, (ContextId, usize)>>,
    /// Context should handle pending signals
    pub pending: VecDeque<u8>,
    /// Context should wake up at specified time
    pub wake: Option<(u64, u64)>,
    /// The architecture specific context
    pub arch: arch::Context,
    /// Kernel FX - used to store SIMD and FPU registers on context switch
    pub kfx: Option<Box<[u8]>>,
    /// Kernel stack
    pub kstack: Option<Box<[u8]>>,
    /// Kernel signal backup
    pub ksig: Option<(arch::Context, Option<Box<[u8]>>, Option<Box<[u8]>>)>,
    /// Restore ksig context on next switch
    pub ksig_restore: bool,
    /// Executable image
    pub image: Vec<SharedMemory>,
    /// User heap
    pub heap: Option<SharedMemory>,
    /// User stack
    pub stack: Option<Memory>,
    /// User signal stack
    pub sigstack: Option<Memory>,
    /// User Thread local storage
    pub tls: Option<Tls>,
    /// User grants
    pub grants: Arc<Mutex<Vec<Grant>>>,
    /// The name of the context
    pub name: Arc<Mutex<Box<[u8]>>>,
    /// The current working directory
    pub cwd: Arc<Mutex<Vec<u8>>>,
    /// The open files in the scheme
    pub files: Arc<Mutex<Vec<Option<FileDescriptor>>>>,
    /// Singal actions
    pub actions: Arc<Mutex<Vec<(SigAction, usize)>>>,
}

