use pic8259_simple::ChainedPics;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub const TIMER_INTERRUPT_ID: u8 = PIC_1_OFFSET;
pub const KEYBOARD_INTERRUPT_ID: u8 = PIC_1_OFFSET + 1;
pub const RTC_INTERRUPT_ID: u8 = PIC_2_OFFSET;
pub const ACPI_INTERRUPT_ID: u8 = PIC_2_OFFSET + 1;
pub const MOUSE_INTERRUPT_ID: u8 = PIC_2_OFFSET + 4;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });