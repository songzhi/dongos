pub mod exception;
pub mod irq;
/// Pause instruction
/// Safe because it is similar to a NOP, and has no memory effects
#[inline(always)]
pub fn pause() {
    unsafe { asm!("pause" : : : : "intel", "volatile"); }
}