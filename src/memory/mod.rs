use bootloader::bootinfo::{MemoryMap, MemoryRegionType, MemoryRegion, BootInfo};
use x86_64::structures::paging::{
    FrameAllocator as SimpleFrameAllocator, FrameDeallocator, Mapper, Page, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};
use spin::{Mutex, Once};
pub use x86_64::{align_down, align_up};

pub mod heap;

pub mod table;
pub mod temporary_page;
pub mod mapper;
pub mod frame_allocator;

pub mod recycle;

pub use self::table::{ActivePageTable, InactivePageTable};
use self::frame_allocator::BumpAllocator;
use self::recycle::RecycleAllocator;

pub static PHYSICAL_MEMORY_OFFSET: Once<u64> = Once::new();
pub static FRAME_ALLOCATOR: Mutex<Option<RecycleAllocator<BumpAllocator>>> = Mutex::new(None);
static mut MEMORY_MAP: Option<&'static MemoryMap> = None;

/// Number of entries per page table
pub const ENTRY_COUNT: usize = 512;

/// Size of pages
pub const PAGE_SIZE: usize = 4096;

/// Init memory module
/// Must be called once, and only once,
pub fn init(boot_info: &'static BootInfo, kernel_start: usize, kernel_end: usize) {
    PHYSICAL_MEMORY_OFFSET.call_once(|| boot_info.physical_memory_offset);
    unsafe { MEMORY_MAP = Some(&boot_info.memory_map); }
    let bump = BumpAllocator::new(kernel_start, kernel_end, MemoryAreaIter::new(MemoryRegionType::Usable));
    *FRAME_ALLOCATOR.lock() = Some(RecycleAllocator::new(bump));
}

pub(crate) fn phys_to_virt(frame: PhysFrame) -> VirtAddr {
    let physical_memory_offset = *PHYSICAL_MEMORY_OFFSET.r#try()
        .expect("PHYSICAL_MEMORY_OFFSET not initialized");
    let phys = frame.start_address().as_u64();
    VirtAddr::new(phys + physical_memory_offset)
}

/// Init memory module after core
/// Must be called once, and only once,
pub unsafe fn init_noncore() {
    if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
        allocator.set_noncore(true)
    } else {
        panic!("frame allocator not initialized");
    }
}

pub fn create_example_mapping(
    active_page_table: &mut ActivePageTable,
    frame_allocator: &mut impl FrameAllocator,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let page: Page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe { active_page_table.map_to(page, frame, flags, frame_allocator) };

    map_to_result.expect("map_to failed").flush();
}

/// Allocate a range of frames
pub fn allocate_frames(count: usize) -> Option<PhysFrame> {
    if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
        allocator.allocate_frames(count)
    } else {
        panic!("frame allocator not initialized");
    }
}

pub trait FrameAllocator: SimpleFrameAllocator<Size4KiB> + FrameDeallocator<Size4KiB> {
    fn set_noncore(&mut self, noncore: bool);
    fn free_frames(&self) -> usize;
    fn used_frames(&self) -> usize;
    fn allocate_frames(&mut self, count: usize) -> Option<PhysFrame>;
    fn deallocate_frames(&mut self, frame: PhysFrame, count: usize);
}

#[derive(Clone)]
pub struct MemoryAreaIter {
    area_type: MemoryRegionType,
    i: usize,
}

impl MemoryAreaIter {
    fn new(area_type: MemoryRegionType) -> Self {
        MemoryAreaIter {
            area_type,
            i: 0,
        }
    }
}

impl Iterator for MemoryAreaIter {
    type Item = &'static MemoryRegion;
    fn next(&mut self) -> Option<Self::Item> {
        while self.i < unsafe { MEMORY_MAP.unwrap().len() } {
            let entry = unsafe { &MEMORY_MAP.unwrap()[self.i] };
            self.i += 1;
            if entry.region_type == self.area_type {
                return Some(entry);
            }
        }
        None
    }
}