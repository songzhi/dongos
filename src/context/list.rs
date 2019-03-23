use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::sync::atomic::Ordering;
use spin::RwLock;

use crate::memory::ActivePageTable;
use crate::syscall::error::{Result, Error, EAGAIN};
use super::context::{Context, ContextId};
use crate::context::contexts;

/// Context list type
pub struct ContextList {
    map: BTreeMap<ContextId, Arc<RwLock<Context>>>,
    next_id: usize,
}

extern "C" fn kthread(func: fn()) {
    func();
}

impl ContextList {
    /// Create a new context list.
    pub fn new() -> Self {
        ContextList {
            map: BTreeMap::new(),
            next_id: 1,
        }
    }

    /// Get the nth context.
    pub fn get(&self, id: ContextId) -> Option<&Arc<RwLock<Context>>> {
        self.map.get(&id)
    }

    /// Get the current context.
    pub fn current(&self) -> Option<&Arc<RwLock<Context>>> {
        self.map.get(&super::CONTEXT_ID.load(Ordering::SeqCst))
    }

    pub fn iter(&self) -> alloc::collections::btree_map::Iter<ContextId, Arc<RwLock<Context>>> {
        self.map.iter()
    }

    /// Create a new context.
    pub fn new_context(&mut self) -> Result<&Arc<RwLock<Context>>> {
        if self.next_id >= super::CONTEXT_MAX_CONTEXTS {
            self.next_id = 1;
        }

        while self.map.contains_key(&ContextId::from(self.next_id)) {
            self.next_id += 1;
        }

        if self.next_id >= super::CONTEXT_MAX_CONTEXTS {
            return Err(Error::new(EAGAIN));
        }

        let id = ContextId::from(self.next_id);
        self.next_id += 1;

        assert!(self.map.insert(id, Arc::new(RwLock::new(Context::new(id)))).is_none());

        Ok(self.map.get(&id).expect("Failed to insert new context. ID is out of bounds."))
    }

    /// Spawn a context from a function.
    pub fn spawn(&mut self, func: extern fn()) -> Result<&Arc<RwLock<Context>>> {
        let context_lock = self.new_context()?;
        {
            let mut context = context_lock.write();
            let mut fx = unsafe { Box::from_raw(crate::HEAP_ALLOCATOR.alloc(Layout::from_size_align_unchecked(512, 16)) as *mut [u8; 512]) };
            for b in fx.iter_mut() {
                *b = 0;
            }
            let mut stack = vec![0; 65_536].into_boxed_slice();
            let offset = stack.len() - mem::size_of::<usize>();
            unsafe {
                let offset = stack.len() - mem::size_of::<usize>();
                let func_ptr = stack.as_mut_ptr().offset(offset as isize);
                let cs_ptr = stack.as_mut_ptr().offset((offset - mem::size_of::<usize>()) as isize);
                *(cs_ptr as *mut usize) = 0x23;
                *(func_ptr as *mut usize) = func as usize;
            }
            println!("new thread func:0x{:X}", func as usize);
            context.arch.set_page_table(unsafe { ActivePageTable::address().as_u64() as usize });
            context.arch.set_fx(fx.as_ptr() as usize);
            context.arch.set_stack(stack.as_ptr() as usize + offset);
            context.kfx = Some(fx);
            context.kstack = Some(stack);
            context.unblock();
        }
        Ok(context_lock)
    }

    pub fn remove(&mut self, id: ContextId) -> Option<Arc<RwLock<Context>>> {
        self.map.remove(&id)
    }
}
