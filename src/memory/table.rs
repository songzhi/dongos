use core::ops::{Deref, DerefMut};
use x86_64::structures::paging::{RecursivePageTable, PhysFrame, PageTable, PageTableFlags as EntryFlags};
use x86_64::PhysAddr;
use bootloader::bootinfo::BootInfo;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::instructions::tlb;
use x86_64::structures::paging::Page;
use spin::Once;

pub static P4_TABLE_ADDR: Once<usize> = Once::new();

use super::temporary_page::TemporaryPage;

pub struct ActivePageTable {
    mapper: RecursivePageTable<'static>,
}

impl Deref for ActivePageTable {
    type Target = RecursivePageTable<'static>;

    fn deref(&self) -> &RecursivePageTable<'static> {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut RecursivePageTable<'static> {
        &mut self.mapper
    }
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        fn init_inner(level_4_table_addr: usize) -> RecursivePageTable<'static> {
            let level_4_table_ptr = level_4_table_addr as *mut PageTable;
            let level_4_table = unsafe { &mut *level_4_table_ptr };
            RecursivePageTable::new(level_4_table).unwrap()
        }
        ActivePageTable {
            mapper: init_inner(*P4_TABLE_ADDR),
        }
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let (p4_frame, flags) = Cr3::read();
        let old_table = InactivePageTable { p4_frame };
        unsafe {
            Cr3::write(new_table.p4_frame, flags);
        }
        old_table
    }

    pub fn flush(&mut self, page: Page) {
        unsafe { tlb::flush(page.start_address()); }
    }

    pub fn flush_all(&mut self) {
        unsafe { tlb::flush_all(); }
    }

    pub fn with<F>(&mut self, table: &mut InactivePageTable, temporary_page: &mut TemporaryPage, f: F)
        where F: FnOnce(&mut RecursivePageTable)
    {
        {
            let backup = Cr3::read().0;

            // map temporary_page to current p4 table
            let p4_table = temporary_page.map_table_frame(backup.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE, self);

            // overwrite recursive mapping
            p4_table[::RECURSIVE_PAGE_PML4].set_frame(table.p4_frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE);
            self.flush_all();

            // execute f in the new context
            f(self);

            // restore recursive mapping to original p4 table
            p4_table[::RECURSIVE_PAGE_PML4].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE);
            self.flush_all();
        }

        temporary_page.unmap(self);
    }

    pub unsafe fn address(&self) -> PhysAddr {
        Cr3::read().0.start_address()
    }
}

pub struct InactivePageTable {
    p4_frame: PhysFrame,
}

impl InactivePageTable {
    pub fn new(frame: PhysFrame, active_table: &mut ActivePageTable, temporary_page: &mut TemporaryPage) -> InactivePageTable {
        {
            let table = temporary_page.map_table_frame(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE, active_table);
            // now we are able to zero the table
            table.zero();
            // set up recursive mapping for the table
            table[::RECURSIVE_PAGE_PML4].set_frame(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE);
        }
        temporary_page.unmap(active_table);
        InactivePageTable { p4_frame: frame }
    }

    pub unsafe fn from_address(cr3: u64) -> InactivePageTable {
        InactivePageTable { p4_frame: PhysFrame::containing_address(PhysAddr::new(cr3)) }
    }

    pub unsafe fn address(&self) -> u64 {
        self.p4_frame.start_address().as_u64()
    }
}