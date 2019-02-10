// The x86-interrupt calling convention leads to the following LLVM error
// when compiled for a Windows target: "offset is not a multiple of 16". This
// happens for example when running `cargo test` on Windows. To avoid this
// problem we skip compilation of this module on Windows.
#![cfg(not(windows))]

use core::sync::atomic::{AtomicUsize, Ordering};
use spin;
use x86_64::structures::idt::ExceptionStackFrame;
use crate::device::pic::*;
use crate::print;
use lazy_static::lazy_static;

//resets to 0 in context::switch()
pub static PIT_TICKS: AtomicUsize = AtomicUsize::new(0);

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut ExceptionStackFrame) {
    //print!('.');
    unsafe { PICS.lock().notify_end_of_interrupt(TIMER_INTERRUPT_ID) }
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut ExceptionStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1));
    }

    let mut keyboard = KEYBOARD.lock();
    let port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe { PICS.lock().notify_end_of_interrupt(KEYBOARD_INTERRUPT_ID) }
}