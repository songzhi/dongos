#![no_std]
#![cfg_attr(not(test), no_main)]
#![allow(dead_code, unused_macros, unused_imports, unused_variables, unused_mut, deprecated)]

use dongos::{exit_qemu, serial_println};
use core::panic::PanicInfo;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    dongos::idt::init();
    unsafe {
        dongos::device::init();
        dongos::device::init_noncore();
    }
    x86_64::instructions::interrupts::int3();

    serial_println!("ok");

    unsafe {
        exit_qemu();
    }
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("failed");

    serial_println!("{}", info);

    unsafe {
        exit_qemu();
    }
    loop {}
}
