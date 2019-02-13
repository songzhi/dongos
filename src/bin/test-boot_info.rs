#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

use bootloader::{bootinfo::BootInfo, entry_point, bootinfo::MemoryRegionType};
use dongos::{exit_qemu, serial_println};
use core::panic::PanicInfo;
entry_point!(kernel_main);
#[cfg(not(test))]
#[no_mangle]
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    dongos::idt::init();
    unsafe {
        dongos::device::init();
        dongos::device::init_noncore();
    }
    x86_64::instructions::int3();
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
