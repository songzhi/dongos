use x86_64::structures::paging::{PhysFrame, PageRangeInclusive, FrameAllocator as SimpleFrameAllocator, FrameDeallocator, Size4KiB};
use x86_64::PhysAddr;
use bootloader::bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};
use core::slice::Iter;
use super::{FrameAllocator, MemoryAreaIter};

pub struct BumpAllocator {
    next_free_frame: PhysFrame,
    current_area: Option<&'static MemoryRegion>,
    areas: MemoryAreaIter,
    kernel_start: PhysFrame,
    kernel_end: PhysFrame,
}

impl BumpAllocator {
    pub fn new(kernel_start: usize, kernel_end: usize, memory_areas: MemoryAreaIter) -> Self {
        let mut allocator = Self {
            next_free_frame: PhysFrame::containing_address(PhysAddr::new(0)),
            current_area: None,
            areas: memory_areas,
            kernel_start: PhysFrame::containing_address(PhysAddr::new(kernel_start as u64)),
            kernel_end: PhysFrame::containing_address(PhysAddr::new(kernel_end as u64)),
        };
        allocator.choose_next_area();
        allocator
    }
    fn choose_next_area(&mut self) {
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

impl FrameAllocator for BumpAllocator {
    fn free_frames(&self) -> usize {
        let mut count = 0;

        for area in self.areas.clone() {
            let start_frame = PhysFrame::containing_address(PhysAddr::new(area.range.start_addr()));
            let end_frame = PhysFrame::containing_address(PhysAddr::new(area.range.end_addr()));
            for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
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
            let start_frame = PhysFrame::containing_address(PhysAddr::new(area.range.start_addr()));
            let end_frame = PhysFrame::containing_address(PhysAddr::new(area.range.end_addr()));
            for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
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

    fn allocate_frames(&mut self, count: usize) -> Option<PhysFrame> {
        if count == 0 {
            None
        } else if let Some(area) = self.current_area {
            // "Clone" the frame to return it if it's free.
            let start_frame = self.next_free_frame.clone();
            let end_frame = self.next_free_frame + (count - 1) as u64;

            // the last frame of the current area
            let current_area_last_frame = PhysFrame::containing_address(PhysAddr::new(area.range.end_addr()));

            if end_frame > current_area_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else if (start_frame >= self.kernel_start && start_frame <= self.kernel_end)
                || (end_frame >= self.kernel_start && end_frame <= self.kernel_end) {
                // `frame` is used by the kernel
                self.next_free_frame = self.kernel_end + 1;
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame += count as u64;
                return Some(start_frame);
            }
            // `frame` was not valid, try it again with the updated `next_free_frame`
            self.allocate_frames(count)
        } else {
            None // no free frames left
        }
    }

    fn deallocate_frames(&mut self, _frame: PhysFrame, _count: usize) {
        //panic!("BumpAllocator::deallocate_frame: not supported: {:?}", frame);
    }
}

impl SimpleFrameAllocator<Size4KiB> for BumpAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.allocate_frames(1)
    }
}

impl FrameDeallocator<Size4KiB> for BumpAllocator {
    fn deallocate_frame(&mut self, frame: PhysFrame) {
        self.deallocate_frames(frame, 1);
    }
}
