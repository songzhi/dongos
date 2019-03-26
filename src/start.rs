use bootloader::{bootinfo::{BootInfo, MemoryRegionType}};

use crate::{println, hlt_loop, interrupt, context};
use super::memory::FRAME_ALLOCATOR;
use crate::context::Status;

pub extern fn context_test() {
    println!("Hello from another thread!");
    hlt_loop();
}

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
        if let Ok(context_lock) = context::contexts_mut().spawn(context_test) {
            let mut context = context_lock.write();
            context.status = Status::Runnable;
        }
    }

    interrupt::enable();

    create_example_mapping(&mut active_page_table, FRAME_ALLOCATOR.lock().as_mut().unwrap());
    // 打印：new！
    unsafe { (0xdeadbeaf900 as *mut u64).write_volatile(0xf021f077f065f04e) };

    println!("It did not crash!");
    hlt_loop();
}