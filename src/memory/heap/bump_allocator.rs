use core::sync::atomic::{AtomicUsize, Ordering};
use core::alloc::{GlobalAlloc, Layout};
use crate::memory::align_up;

/// A simple allocator that allocates memory linearly and ignores freed memory.
#[derive(Debug)]
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: AtomicUsize,
}

impl BumpAllocator {
    pub const fn new(heap_start: usize, heap_end: usize) -> Self {
        // NOTE: requires adding #![feature(const_atomic_usize_new)] to lib.rs
        Self { heap_start, heap_end, next: AtomicUsize::new(heap_start) }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        loop {
            // load current state of the `next` field
            let current_next = self.next.load(Ordering::Relaxed);
            let alloc_start = align_up(current_next as u64, layout.align() as u64);
            let alloc_end = alloc_start.saturating_add(layout.size() as u64) as usize;

            if alloc_end <= self.heap_end {
                // update the `next` pointer if it still has the value `current_next`
                let next_now = self.next.compare_and_swap(current_next, alloc_end,
                                                          Ordering::Relaxed);
                if next_now == current_next {
                    // next address was successfully updated, allocation succeeded
                    return alloc_start as *mut u8;
                }
            } else {
                return 0 as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // do nothing, leak memory
    }
}