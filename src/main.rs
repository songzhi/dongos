#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

use dongos::println;
use core::panic::PanicInfo;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    use dongos::interrupts::PICS;
    use x86_64::structures::paging::PageTable;

    println!("Hello World{}", "!");

    dongos::gdt::init();
    dongos::interrupts::init_idt();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let level_4_table_ptr = 0xffff_ffff_ffff_f000 as *const PageTable;
    let level_4_table = unsafe { &*level_4_table_ptr };
    for i in 0..10 {
        println!("Entry {}: {:?}", i, level_4_table[i]);
    }

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
