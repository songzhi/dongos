#[cfg(not(test))]
pub mod bump_allocator;

pub const HEAP_SIZE: usize = 100 * 1024;
// 100 KiB
pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_END: usize = HEAP_START + HEAP_SIZE;

/// Error handler for allocation errors
#[cfg(not(test))]
mod alloc_error {
    use alloc::alloc::Layout;
    use crate::println;

    #[alloc_error_handler]
    pub fn handle_alloc_error(layout: Layout) -> ! {
        println!("Allocation Error");
        println!("{:#?}", layout);
        panic!();
    }
}