#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

use dongos::println;
use core::panic::PanicInfo;
use bootloader::{bootinfo::BootInfo, entry_point};

entry_point!(kernel_main);

#[cfg(not(test))]
#[no_mangle]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use dongos::memory::{self, create_example_mapping};

    println!("Hello World{}", "!");

    dongos::gdt::init();
    dongos::idt::init();
    x86_64::instructions::interrupts::enable();
    unsafe {
        dongos::device::init();
        dongos::device::init_noncore();
    };

    let mut recursive_page_table = unsafe { memory::init(boot_info.p4_table_addr as usize) };
    let mut frame_allocator = memory::init_frame_allocator(&boot_info.memory_map);

    create_example_mapping(&mut recursive_page_table, &mut frame_allocator);
    unsafe { (0xdeadbeaf900 as *mut u64).write_volatile(0xf021f077f065f04e) };


    let time = {
        use dongos::{
            syscall::{
                data::RtcDateTime,
            },
            device::rtc::Rtc,
        };

        let mut t = Rtc::new();
        t.date_time()
    };
    println!("{:#?}", time);

    println!("It did not crash!");
    dongos::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    dongos::hlt_loop();
}
