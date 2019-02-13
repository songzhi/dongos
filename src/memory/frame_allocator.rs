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
    fn free_frames(&self) -> usize {
        let mut count = 0;

        for area in self.areas.clone() {
            let start_frame = Frame::containing_address(PhysicalAddress::new(area.base_addr as usize));
            let end_frame = Frame::containing_address(PhysicalAddress::new((area.base_addr + area.length - 1) as usize));
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                if frame >= self.kernel_start && frame <= self.kernel_end {
                    // Inside of kernel range
                } else if frame >= self.next_free_frame {
                    // Frame is in free range
                    count += 1;
                } else {
                    // Inside of used range
                }
            }
        }

        count
    }

    fn used_frames(&self) -> usize {
        let mut count = 0;

        for area in self.areas.clone() {
            let start_frame = Frame::containing_address(PhysicalAddress::new(area.base_addr as usize));
            let end_frame = Frame::containing_address(PhysicalAddress::new((area.base_addr + area.length - 1) as usize));
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                if frame >= self.kernel_start && frame <= self.kernel_end {
                    // Inside of kernel range
                    count += 1
                } else if frame >= self.next_free_frame {
                    // Frame is in free range
                } else {
                    count += 1;
                }
            }
        }

        count
    }
}
