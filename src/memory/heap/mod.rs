#[cfg(not(test))]
pub mod bump_allocator;

use x86_64::structures::paging::Page;
use x86_64::VirtAddr;

use super::table::ActivePageTable;
use super::mapper::MapperFlushAll;
use crate::HEAP_ALLOCATOR;

pub const HEAP_SIZE: usize = 100 * 1024;
// 100 KiB
pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_END: usize = HEAP_START + HEAP_SIZE;

unsafe fn map_heap(active_table: &mut ActivePageTable, offset: usize, size: usize) {
    let mut flush_all = MapperFlushAll::new();

    let heap_start_page = Page::containing_address(VirtAddr::new(offset as u64));
    let heap_end_page = Page::containing_address(VirtAddr::new((offset + size - 1) as u64));
    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        let result = active_table.map(page, EntryFlags::PRESENT | EntryFlags::GLOBAL | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE);
        flush_all.consume(result);
    }

    flush_all.flush(active_table);
}

pub unsafe fn init(active_table: &mut ActivePageTable) {
    let offset = crate::KERNEL_HEAP_OFFSET;
    let size = crate::KERNEL_HEAP_SIZE;

    // Map heap pages
    map_heap(active_table, offset, size);

    // Initialize global heap
    unsafe { HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE); }
}

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