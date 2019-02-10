use alloc::sync::{Arc, Weak};
use spin::Mutex;
use x86_64::{
    VirtAddr,
    structures::{
        paging::{
            PageRangeInclusive,
            Page,
            MapperFlush,
            PageTableFlags as EntryFlags,
        }
    },
};
use core::intrinsics;

use crate::memory::{ActivePageTable, InactivePageTable};
use crate::memory::temporary_page::TemporaryPage;

#[derive(Clone, Debug)]
pub enum SharedMemory {
    Owned(Arc<Mutex<Memory>>),
    Borrowed(Weak<Mutex<Memory>>),
}

impl SharedMemory {
    pub fn with<F, T>(&self, f: F) -> T where F: FnOnce(&mut Memory) -> T {
        match *self {
            SharedMemory::Owned(ref memory_lock) => {
                let mut memory = memory_lock.lock();
                f(&mut *memory)
            }
            SharedMemory::Borrowed(ref memory_weak) => {
                let memory_lock = memory_weak.upgrade().expect("SharedMemory::Borrowed no longer valid");
                let mut memory = memory_lock.lock();
                f(&mut *memory)
            }
        }
    }

    pub fn borrow(&self) -> SharedMemory {
        match *self {
            SharedMemory::Owned(ref memory_lock) => SharedMemory::Borrowed(Arc::downgrade(memory_lock)),
            SharedMemory::Borrowed(ref memory_lock) => SharedMemory::Borrowed(memory_lock.clone())
        }
    }
}

#[derive(Debug)]
pub struct Memory {
    start: VirtAddr,
    size: usize,
    flags: EntryFlags,
}

impl Memory {
    pub fn new(start: VirtAddr, size: usize, flags: EntryFlags, clear: bool) -> Self {
        let mut memory = Memory {
            start,
            size,
            flags,
        };

        memory.map(clear);

        memory
    }

    pub fn to_shared(self) -> SharedMemory {
        SharedMemory::Owned(Arc::new(Mutex::new(self)))
    }

    pub fn start_address(&self) -> VirtAddr {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn flags(&self) -> EntryFlags {
        self.flags
    }

    pub fn pages(&self) -> PageRangeInclusive {
        let start_page = Page::containing_address(self.start);
        let end_page = Page::containing_address(VirtAddr::new(self.start.get() + self.size - 1));
        Page::range_inclusive(start_page, end_page)
    }

    fn map(&mut self, clear: bool) {
        let mut active_table = unsafe { ActivePageTable::new() };

        for page in self.pages() {
            let result = active_table.map(page, self.flags);
            MapperFlush::flush(result);
        }
        if clear {
            assert!(self.flags.contains(EntryFlags::WRITABLE));
            unsafe {
                intrinsics::write_bytes(self.start_address().get() as *mut u8, 0, self.size);
            }
        }
    }

    fn unmap(&mut self) {
        let mut active_table = unsafe { ActivePageTable::new() };
        for page in self.pages() {
            let result = active_table.unmap(page);
            MapperFlush::flush(result);
        }
    }

    /// A complicated operation to move a piece of memory to a new page table
    /// It also allows for changing the address at the same time
    pub fn move_to(&mut self, new_start: VirtAddr, new_table: &mut InactivePageTable, temporary_page: &mut TemporaryPage) {
        let mut active_table = unsafe { ActivePageTable::new() };

        for page in self.pages() {
            let (result, frame) = active_table.unmap_return(page, false);
            MapperFlush::flush(result);

            active_table.with(new_table, temporary_page, |mapper| {
                let new_page = Page::containing_address(VirtAddr::new(page.start_address().get() - self.start.get() + new_start.get()));
                let result = mapper.map_to(new_page, frame, self.flags);
                // This is not the active table, so the flush can be ignored
                unsafe { result.ignore(); }
            });
        }

        self.start = new_start;
    }

    pub fn remap(&mut self, new_flags: EntryFlags) {
        let mut active_table = unsafe { ActivePageTable::new() };

        for page in self.pages() {
            let result = active_table.remap(page, new_flags);
            MapperFlush::flush(result);
        }

        self.flags = new_flags;
    }

    pub fn resize(&mut self, new_size: usize, clear: bool) {
        let mut active_table = unsafe { ActivePageTable::new() };

        //TODO: Calculate page changes to minimize operations
        if new_size > self.size {

            let start_page = Page::containing_address(VirtAddr::new(self.start.get() + self.size));
            let end_page = Page::containing_address(VirtAddr::new(self.start.get() + new_size - 1));
            for page in Page::range_inclusive(start_page, end_page) {
                if active_table.translate_page(page).is_none() {
                    let result = active_table.map(page, self.flags);
                    MapperFlush::flush(result);
                }
            }

            if clear {
                unsafe {
                    intrinsics::write_bytes((self.start.get() + self.size) as *mut u8, 0, new_size - self.size);
                }
            }
        } else if new_size < self.size {

            let start_page = Page::containing_address(VirtAddr::new(self.start.get() + new_size));
            let end_page = Page::containing_address(VirtAddr::new(self.start.get() + self.size - 1));
            for page in Page::range_inclusive(start_page, end_page) {
                if active_table.translate_page(page).is_some() {
                    let result = active_table.unmap(page);
                    MapperFlush::flush(result);
                }
            }

        }

        self.size = new_size;
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        self.unmap();
    }
}

#[derive(Debug)]
pub struct Tls {
    pub master: VirtAddr,
    pub file_size: usize,
    pub mem: Memory,
    pub offset: usize,
}

impl Tls {
    /// Load TLS data from master
    pub unsafe fn load(&mut self) {
        intrinsics::copy(
            self.master.as_u64() as *const u8,
            (self.mem.start_address().as_u64() + self.offset) as *mut u8,
            self.file_size,
        );
    }
}