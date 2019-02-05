#![cfg_attr(not(test), no_std)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(const_fn)]
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
#[macro_use]
/// Shared data structures
pub mod common;

pub use self::start::kernel_main;
use linked_list_allocator::LockedHeap;

// Heap allocator (disabled during testing)
#[cfg(not(test))]
#[cfg_attr(not(test), global_allocator)]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

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