use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, Page, PageTable, PhysFrame, RecursivePageTable, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};
use spin::Mutex;
use core::mem;
use alloc::boxed::Box;
pub use x86_64::{align_down, align_up};

#[cfg(not(test))]
pub mod heap;

pub mod table;
pub mod temporary_page;
pub mod mapper;
pub mod frame_allocator;

#[cfg(not(test))]
pub use self::heap::bump_allocator::BumpAllocator;
#[cfg(not(test))]
pub use self::heap::{HEAP_START, HEAP_END, HEAP_SIZE};
pub use self::table::{ActivePageTable, InactivePageTable, P4_TABLE_ADDR};

pub static FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator<impl Iterator<Item=PhysFrame>>>> = Mutex::new(None);
/// Number of entries per page table
pub const ENTRY_COUNT: usize = 512;

/// Size of pages
pub const PAGE_SIZE: usize = 4096;

/// Creates a RecursivePageTable instance from the level 4 address.
///
/// This function is unsafe because it can break memory safety if an invalid
/// address is passed.
pub unsafe fn init(level_4_table_addr: usize) -> RecursivePageTable<'static> {
    /// Rust currently treats the whole body of unsafe functions as an unsafe
    /// block, which makes it difficult to see which operations are unsafe. To
    /// limit the scope of unsafe we use a safe inner function.
    fn init_inner(level_4_table_addr: usize) -> RecursivePageTable<'static> {
        let level_4_table_ptr = level_4_table_addr as *mut PageTable;
        let level_4_table = unsafe { &mut *level_4_table_ptr };
        RecursivePageTable::new(level_4_table).unwrap()
    }
    init_inner(level_4_table_addr)
}

/// Create a FrameAllocator from the passed memory map
pub fn init_frame_allocator(
    memory_map: &'static MemoryMap,
) {
    fn init_inner(
        memory_map: &'static MemoryMap,
    ) -> BootInfoFrameAllocator<Iter<PhysFrame>> {
        // get usable regions from memory map
        let regions = memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.into_iter().step_by(4096));
        // create `PhysFrame` types from the start addresses
        let frames = frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)));
        let frames: Vec<PhysFrame> = frames.collect();
        BootInfoFrameAllocator { frames: frames.iter() }
    }
    *FRAME_ALLOCATOR.lock() = Some(init_inner(memory_map));
}

/// Returns the physical address for the given virtual address, or `None` if
/// the virtual address is not mapped.
pub fn translate_addr(addr: u64, recursive_page_table: &RecursivePageTable) -> Option<PhysAddr> {
    let addr = VirtAddr::new(addr);
    let page: Page = Page::containing_address(addr);

    // perform the translation
    let frame = recursive_page_table.translate_page(page);
    frame.map(|frame| frame.start_address() + u64::from(addr.page_offset()))
}

pub fn create_example_mapping(
    recursive_page_table: &mut RecursivePageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let page: Page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe { recursive_page_table.map_to(page, frame, flags, frame_allocator) };

    let heap_start_page = Page::containing_address(VirtAddr::new(HEAP_START as u64));
    let heap_end_page = Page::containing_address(VirtAddr::new((HEAP_END - 1) as u64));

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        unsafe { recursive_page_table.map_to(page, frame_allocator.allocate_frame().unwrap(), flags, frame_allocator); }
    }

    map_to_result.expect("map_to failed").flush();
}

/// Allocate a range of frames
pub fn allocate_frame() -> Option<PhysFrame> {
    if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
        allocator.allocate_frame()
    } else {
        panic!("frame allocator not initialized");
    }
}

pub trait FrameAllocator {
    fn free_frames(&self) -> usize;
    fn used_frames(&self) -> usize;
    fn allocate_frames(&mut self, size: usize) -> Option<PhysFrame>;
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.allocate_frames(1)
    }
    fn deallocate_frames(&mut self, frame: Frame, size: usize);
}