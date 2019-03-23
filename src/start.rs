use bootloader::{bootinfo::{BootInfo, MemoryRegionType}};

use crate::{println, context};
use super::memory::FRAME_ALLOCATOR;
use crate::syscall::arch::syscall0;
use x86_64::VirtAddr;
use x86_64::structures::paging::mapper::{Mapper, MapperAllSizes};
use x86_64::structures::paging::{Page, Size4KiB};


#[no_mangle]
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use crate::memory::{self, create_example_mapping, ActivePageTable, heap};

    println!("Hello World{}", "!");

    crate::gdt::init();
    crate::idt::init();
    unsafe {
        crate::device::init();
    };


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

    unsafe {
        // Initialize all of the non-core devices not otherwise needed to complete initialization
        crate::device::init_noncore();

        // Initialize memory functions after core has loaded
        memory::init_noncore();
    }
    context::init();

    {
        extern "C" fn print_hello() {}
        let virt = VirtAddr::new(print_hello as u64);
        use x86_64::structures::paging::PageTableFlags as Flags;
        let page = Page::<Size4KiB>::containing_address(virt);

        context::contexts_mut().spawn(print_hello).expect("spawn failed");
    }
    x86_64::instructions::interrupts::enable();

    create_example_mapping(&mut active_page_table, FRAME_ALLOCATOR.lock().as_mut().unwrap());
    // 打印：new！
    unsafe { (0xdeadbeaf900 as *mut u64).write_volatile(0xf021f077f065f04e) };

    println!("It did not crash!");
    crate::hlt_loop();
}