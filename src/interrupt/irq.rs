use core::sync::atomic::AtomicUsize;
use spin;
use x86_64::structures::idt::InterruptStackFrame;
use crate::device::pic::*;
use crate::{print, time};
use lazy_static::lazy_static;
use core::sync::atomic::Ordering;
use crate::context;

//resets to 0 in context::switch()
pub static PIT_TICKS: AtomicUsize = AtomicUsize::new(0);

unsafe fn irq_trigger(interrupt_id: u8) {
    PICS.lock().notify_end_of_interrupt(interrupt_id);
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    const PIT_RATE: u64 = 2_250_286;

    {
        let mut offset = time::OFFSET.lock();
        let sum = offset.1 + PIT_RATE;
        offset.1 = sum % 1_000_000_000;
        offset.0 += sum / 1_000_000_000;
    }

//    timeout::trigger();
    if PIT_TICKS.fetch_add(1, Ordering::SeqCst) >= 10 {
        let _ = unsafe { context::switch() };
    }
    unsafe { irq_trigger(InterruptIndex::Timer.as_u8()); }
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe { irq_trigger(InterruptIndex::Keyboard.as_u8()); }
}