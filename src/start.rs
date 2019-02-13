use bootloader::{bootinfo::{BootInfo, MemoryRegionType}};

use crate::{println, print};
use super::HEAP_ALLOCATOR;
use super::memory::{P4_TABLE_ADDR, FRAME_ALLOCATOR};

#[cfg(not(test))]
#[no_mangle]
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use crate::memory::{self, create_example_mapping, ActivePageTable, heap};

    println!("Hello World{}", "!");

    crate::gdt::init();
    crate::idt::init();
    unsafe {
        crate::device::init();
        crate::device::init_noncore();
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

    create_example_mapping(&mut active_page_table, FRAME_ALLOCATOR.lock().as_mut().unwrap());
    // 打印：new！
    unsafe { (0xdeadbeaf900 as *mut u64).write_volatile(0xf021f077f065f04e) };

    println!("It did not crash!");
    crate::hlt_loop();
}