use bootloader::{bootinfo::BootInfo};

use crate::{println, print};
use super::HEAP_ALLOCATOR;
use super::memory::{HEAP_START, HEAP_SIZE};

#[cfg(not(test))]
#[no_mangle]
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use crate::memory::{self, create_example_mapping};

    println!("Hello World{}", "!");

    crate::gdt::init();
    crate::idt::init();
    unsafe {
        crate::device::init();
        crate::device::init_noncore();
    };
    x86_64::instructions::interrupts::enable();

    let mut recursive_page_table = unsafe { memory::init(boot_info.p4_table_addr as usize) };
    let mut frame_allocator = memory::init_frame_allocator(&boot_info.memory_map);
    create_example_mapping(&mut recursive_page_table, &mut frame_allocator);

    unsafe { HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE); }
    // 打印：new！
    unsafe { (0xdeadbeaf900 as *mut u64).write_volatile(0xf021f077f065f04e) };

    {
        use alloc::boxed::Box;
        use alloc::prelude::*;
        let mut heap_test = Box::new(42);
        *heap_test -= 15;
        let heap_test2 = Box::new("hello");
        println!("{:?} {:?}", heap_test, heap_test2);

        let mut vec_test = vec![1, 2, 3, 4, 5, 6, 7];
        vec_test[3] = 42;
        for i in &vec_test {
            print!("{} ", i);
        }
    }

    println!("It did not crash!");
    crate::hlt_loop();
}