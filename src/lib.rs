#![cfg_attr(not(test), no_std)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(const_fn, core_intrinsics, thread_local, naked_functions)]
#![feature(alloc_error_handler)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

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
pub mod start;
pub mod context;
pub mod consts;
#[macro_use]
pub mod common;
/// Synchronization primitives
pub mod sync;

pub use consts::*;
pub use self::start::kernel_main;
use linked_list_allocator::LockedHeap;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::panic::PanicInfo;

pub fn init() {
    gdt::init();
    idt::init();
    x86_64::instructions::interrupts::enable();
}

// Heap allocator (disabled during testing)
#[cfg_attr(not(test), global_allocator)]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

/// A unique number that identifies the current CPU - used for scheduling
static CPU_ID: AtomicUsize = AtomicUsize::new(0);

/// Get the current CPU's scheduling ID
#[inline(always)]
pub fn cpu_id() -> usize {
    CPU_ID.load(Ordering::Relaxed)
}

/// The count of all CPUs that can have work scheduled
static CPU_COUNT: AtomicUsize = AtomicUsize::new(1);

/// Get the number of CPUs currently active
#[inline(always)]
pub fn cpu_count() -> usize {
    CPU_COUNT.load(Ordering::Relaxed)
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo xtest`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}
