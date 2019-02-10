#![cfg_attr(not(test), no_std)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(const_fn, core_intrinsics, impl_trait_in_bindings, thread_local, naked_functions)]
#![feature(alloc, allocator_api, alloc_error_handler)]

#[cfg(not(test))]
#[macro_use]
extern crate alloc;

pub mod serial;
#[macro_use]
pub mod vga_buffer;
pub mod interrupt;
pub mod gdt;
pub mod idt;
pub mod memory;
pub mod time;
pub mod syscall;
pub mod device;
pub mod thread;
pub mod start;
pub mod context;
pub mod consts;
#[macro_use]
/// Shared data structures
pub mod common;
/// Synchronization primitives
pub mod sync;

pub use consts::*;
pub use self::start::kernel_main;
use linked_list_allocator::LockedHeap;
use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

// Heap allocator (disabled during testing)
#[cfg(not(test))]
#[cfg_attr(not(test), global_allocator)]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

/// A unique number that identifies the current CPU - used for scheduling
#[thread_local]
static CPU_ID: AtomicUsize = ATOMIC_USIZE_INIT;

/// Get the current CPU's scheduling ID
#[inline(always)]
pub fn cpu_id() -> usize {
    CPU_ID.load(Ordering::Relaxed)
}

/// The count of all CPUs that can have work scheduled
static CPU_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

/// Get the number of CPUs currently active
#[inline(always)]
pub fn cpu_count() -> usize {
    CPU_COUNT.load(Ordering::Relaxed)
}

pub unsafe fn exit_qemu() {
    use x86_64::instructions::port::Port;

    let mut port = Port::<u32>::new(0xf4);
    port.write(0);
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}