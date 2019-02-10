//! Temporarily map a page
//! From [Phil Opp's Blog](http://os.phil-opp.com/remap-the-kernel.html)
use x86_64::{
    VirtAddr,
    structures::paging::{Page, Size4KiB, PhysFrame, PageTableFlags, PageTable, Mapper}, };
use super::ActivePageTable;
use super::FRAME_ALLOCATOR;
use core::borrow::BorrowMut;
pub struct TemporaryPage {
    page: Page<Size4KiB>,
}

impl TemporaryPage {
    pub fn new(page: Page) -> TemporaryPage {
        TemporaryPage {
            page,
        }
    }

    pub fn start_address(&self) -> VirtAddr {
        self.page.start_address()
    }

    /// Maps the temporary page to the given frame in the active table.
    /// Returns the start address of the temporary page.
    pub fn map(&mut self, frame: PhysFrame, flags: PageTableFlags, active_table: &mut ActivePageTable) -> VirtAddr {
        assert!(active_table.translate_page(self.page).is_none(), "temporary page is already mapped");
        let result = unsafe { active_table.map_to(self.page, frame, flags, FRAME_ALLOCATOR.lock().as_mut().unwrap()).unwrap() };

        result.flush();
        self.page.start_address()
    }

    /// Maps the temporary page to the given page table frame in the active
    /// table. Returns a reference to the now mapped table.
    pub fn map_table_frame(&mut self, frame: PhysFrame, flags: PageTableFlags, active_table: &mut ActivePageTable) -> &mut PageTable {
        unsafe { &mut *(self.map(frame, flags, active_table).as_u64() as *mut PageTable) }
    }

    /// Unmaps the temporary page in the active table.
    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        let (_, result) = active_table.unmap(self.page).unwrap();
        result.flush();
    }
}
