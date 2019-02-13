use x86_64::structures::paging::{PhysFrame, PageRangeInclusive, FrameAllocator, FrameDeallocator};
use x86_64::PhysAddr;
use bootloader::bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};
use core::slice::Iter;

pub struct BumpAllocator {
    next_free_frame: PhysFrame,
    current_area: Option<&'static MemoryRegion>,
    areas: Iter<'static, MemoryRegion>,
    kernel_start: PhysFrame,
    kernel_end: PhysFrame,
}

impl BumpAllocator {
    pub fn new(kernel_start: usize, kernel_end: usize, memory_areas: Iter<'static, MemoryRegion>) -> Self {
        let mut allocator = Self {
            next_free_frame: PhysFrame::containing_address(PhysAddr::new(0)),
            current_area: None,
            areas: memory_areas,
            kernel_start: PhysFrame::containing_address(PhysAddr::new(kernel_start as u64)),
            kernel_end: PhysFrame::containing_address(PhysAddr::new(kernel_end as u64)),
        };
        allocator.choose_next_region();
        allocator
    }
    fn choose_next_region(&mut self) {
        self.current_area = self.areas.clone().filter(|area| {
            let address = area.range.end_addr();
            PhysFrame::containing_address(PhysAddr::new(address)) >= self.next_free_frame
        }).min_by_key(|area| area.range.start_addr());

        if let Some(area) = self.current_area {
            let start_frame = PhysFrame::containing_address(PhysAddr::new(area.range.start_addr()));
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}
