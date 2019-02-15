//! Recycle allocator
//! Uses freed frames if possible, then uses inner allocator

use alloc::vec::Vec;
use x86_64::structures::paging::{PhysFrame, FrameAllocator as SimpleFrameAllocator, FrameDeallocator, Size4KiB};
use super::FrameAllocator;
use x86_64::PhysAddr;

pub struct RecycleAllocator<T: FrameAllocator> {
    inner: T,
    noncore: bool,
    free: Vec<(usize, usize)>,
}

impl<T: FrameAllocator> RecycleAllocator<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            noncore: false,
            free: Vec::new(),
        }
    }

    fn free_count(&self) -> usize {
        let mut count = 0;
        for free in self.free.iter() {
            count += free.1;
        }
        count
    }

    fn merge(&mut self, address: usize, count: usize) -> bool {
        for i in 0..self.free.len() {
            let changed = {
                let free = &mut self.free[i];
                if address + count * 4096 == free.0 {
                    free.0 = address;
                    free.1 += count;
                    true
                } else if free.0 + free.1 * 4096 == address {
                    free.1 += count;
                    true
                } else {
                    false
                }
            };

            if changed {
                //TODO: Use do not use recursion
                let (address, count) = self.free[i];
                if self.merge(address, count) {
                    self.free.remove(i);
                }
                return true;
            }
        }

        false
    }
}

impl<T: FrameAllocator> FrameAllocator for RecycleAllocator<T> {
    fn free_frames(&self) -> usize {
        self.inner.free_frames() + self.free_count()
    }

    fn used_frames(&self) -> usize {
        self.inner.used_frames() - self.free_count()
    }

    fn allocate_frames(&mut self, count: usize) -> Option<PhysFrame> {
        let mut small_i = None;
        {
            let mut small = (0, 0);
            for i in 0..self.free.len() {
                let free = self.free[i];
                // Later entries can be removed faster
                if free.1 >= count {
                    if free.1 <= small.1 || small_i.is_none() {
                        small_i = Some(i);
                        small = free;
                    }
                }
            }
        }

        if let Some(i) = small_i {
            let (address, remove) = {
                let free = &mut self.free[i];
                free.1 -= count;
                (free.0 + free.1 * 4096, free.1 == 0)
            };

            if remove {
                self.free.remove(i);
            }

            //println!("Restoring frame {:?}, {}", frame, count);
            Some(PhysFrame::containing_address(PhysAddr::new(address as u64)))
        } else {
            //println!("No saved frames {}", count);
            self.inner.allocate_frames(count)
        }
    }
    fn deallocate_frames(&mut self, frame: PhysFrame, count: usize) {
        if self.noncore {
            let address = frame.start_address().get();
            if !self.merge(address, count) {
                self.free.push((address, count));
            }
        } else {
            //println!("Could not save frame {:?}, {}", frame, count);
            self.inner.deallocate_frames(frame, count);
        }
    }
}

impl<T: FrameAllocator> SimpleFrameAllocator<Size4KiB> for RecycleAllocator<T> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.allocate_frames(1)
    }
}

impl<T: FrameAllocator> FrameDeallocator<Size4KiB> for RecycleAllocator<T> {
    fn deallocate_frame(&mut self, frame: PhysFrame) {
        self.deallocate_frames(frame, 1);
    }
}