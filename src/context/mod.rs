//! # Context management
//!
//! For resources on contexts, please consult [wikipedia](https://en.wikipedia.org/wiki/Context_switch) and  [osdev](https://wiki.osdev.org/Context_Switching)
use alloc::boxed::Box;
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::Ordering;
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub use self::context::{Context, ContextId, Status, WaitpidKey};
pub use self::list::ContextList;
pub use self::switch::switch;

mod context;
mod list;
mod switch;
pub mod memory;
pub mod signal;
#[path = "arch/x86_64.rs"]
mod arch;

/// Limit on number of contexts
pub const CONTEXT_MAX_CONTEXTS: usize = (isize::max_value() as usize) - 1;

/// Maximum context files
pub const CONTEXT_MAX_FILES: usize = 65_536;

/// Contexts list
static CONTEXTS: Once<RwLock<ContextList>> = Once::new();


static CONTEXT_ID: context::AtomicContextId = context::AtomicContextId::default();

pub fn init() {
    let mut contexts = contexts_mut();
    let context_lock = contexts.new_context().expect("could not initialize first context");
    let mut context = context_lock.write();
    let mut fx = unsafe { Box::from_raw(crate::HEAP_ALLOCATOR.alloc(Layout::from_size_align_unchecked(512, 16)) as *mut [u8; 512]) };
    for b in fx.iter_mut() {
        *b = 0;
    }

    context.arch.set_fx(fx.as_ptr() as usize);
    context.kfx = Some(fx);
    context.status = Status::Runnable;
    context.running = true;
    context.cpu_id = Some(crate::cpu_id());
    CONTEXT_ID.store(context.id, Ordering::SeqCst);
}

/// Initialize contexts, called if needed
fn init_contexts() -> RwLock<ContextList> {
    RwLock::new(ContextList::new())
}

/// Get the global schemes list, const
pub fn contexts() -> RwLockReadGuard<'static, ContextList> {
    //call once will init_contexts only once during the kernel's exececution, otherwise it will return the current context via a
    //cache.
    CONTEXTS.call_once(init_contexts).read()
}

/// Get the global schemes list, mutable
pub fn contexts_mut() -> RwLockWriteGuard<'static, ContextList> {
    CONTEXTS.call_once(init_contexts).write()
}

pub fn context_id() -> ContextId {
    CONTEXT_ID.load(Ordering::SeqCst)
}