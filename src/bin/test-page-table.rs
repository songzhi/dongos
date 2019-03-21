#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

use bootloader::{bootinfo::BootInfo, entry_point, bootinfo::MemoryRegionType};
use dongos::{exit_qemu, serial_println};
use core::panic::PanicInfo;
use x86_64::structures::paging::mapper::MapperAllSizes;
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

    use x86_64::structures::paging::{Page, PhysFrame};
    use x86_64::{VirtAddr, PhysAddr};
    use x86_64::structures::paging::PageTableFlags as Flags;
    use dongos::memory::FRAME_ALLOCATOR;
    use x86_64::structures::paging::Mapper;
    use x86_64::structures::paging::FrameAllocator;

    let page: Page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    let frame = FRAME_ALLOCATOR.lock().as_mut().unwrap().allocate_frame().unwrap();
    let flags = Flags::PRESENT | Flags::WRITABLE;
    unsafe {
        active_page_table.map_to(page, frame, flags, FRAME_ALLOCATOR.lock().as_mut().unwrap());
    }
    assert_eq!(active_page_table.translate_addr(page.start_address()).unwrap(), frame.start_address());
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
