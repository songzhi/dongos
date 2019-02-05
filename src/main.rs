#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

use dongos::*;
use core::panic::PanicInfo;
use bootloader::entry_point;

#[macro_use]
extern crate dongos;

use dongos::kernel_main;
entry_point!(kernel_main);


/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    dongos::hlt_loop();
}
