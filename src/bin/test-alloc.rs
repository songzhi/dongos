#![no_std]
#![cfg_attr(not(test), no_main)]
#![allow(dead_code, unused_macros, unused_imports, unused_variables, unused_mut, deprecated)]
#![feature(alloc)]

#[macro_use]
extern crate alloc;

use bootloader::{bootinfo::BootInfo, entry_point, bootinfo::MemoryRegionType};
use dongos::{exit_qemu, serial_println};
use core::panic::PanicInfo;
entry_point!(kernel_main);
#[cfg(not(test))]
#[no_mangle]
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use dongos::memory::{self, create_example_mapping, ActivePageTable, heap};
    dongos::gdt::init();
    dongos::idt::init();
    unsafe {
        dongos::device::init();
        dongos::device::init_noncore();
    };
    x86_64::instructions::interrupts::enable();

    let kernel_end = {
        let end_area = boot_info.memory_map
            .iter()
            .filter(|area| area.region_type == MemoryRegionType::Kernel)
            .last().unwrap();
        end_area.range.end_addr() as usize
    };
    memory::init(boot_info, 0, kernel_end);

    let mut active_page_table = unsafe {
        let mut active_page_table = ActivePageTable::new();
        heap::init(&mut active_page_table);
        active_page_table
    };

    use alloc::vec;
    let v = vec![1, 2, 3, 4];
    assert_eq!(*v.last().unwrap(), 4);

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
