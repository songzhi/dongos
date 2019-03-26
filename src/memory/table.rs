use core::ops::{Deref, DerefMut};
use x86_64::structures::paging::{MappedPageTable, PhysFrame, PageTable, Size4KiB, PageTableFlags as EntryFlags};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::control::Cr3;
use x86_64::instructions::tlb;
use x86_64::structures::paging::Page;


pub use x86_64::structures::paging::{Mapper, FrameAllocator};
use super::{FRAME_ALLOCATOR, allocate_frames, PHYSICAL_MEMORY_OFFSET, phys_to_virt};
use super::mapper::MapperFlush;

type MappedTable = MappedPageTable<'static, fn(PhysFrame) -> *mut PageTable>;

pub struct ActivePageTable {
    mapper: MappedTable,
}

impl Deref for ActivePageTable {
    type Target = MappedTable;

    fn deref(&self) -> &MappedTable {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut MappedTable {
        &mut self.mapper
    }
}

/// Returns a mutable reference to the level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn get_level_4_table(p4_frame: PhysFrame) -> &'static mut PageTable {
    let physical_memory_offset = *PHYSICAL_MEMORY_OFFSET.r#try()
        .expect("PHYSICAL_MEMORY_OFFSET not initialized");
    let phys = p4_frame.start_address();
    let virt = VirtAddr::new(phys.as_u64() + physical_memory_offset);
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

unsafe fn active_level_4_table() -> &'static mut PageTable {
    let (p4_frame, _) = Cr3::read();
    get_level_4_table(p4_frame)
}


impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        let level_4_table = active_level_4_table();
        ActivePageTable {
            mapper: MappedTable::new(level_4_table, |frame| phys_to_virt(frame).as_mut_ptr()),
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
        tlb::flush(page.start_address());
    }

    pub fn flush_all(&mut self) {
        tlb::flush_all();
    }

    pub fn with<F>(&mut self, table: &mut InactivePageTable, f: F)
        where F: FnOnce(&mut MappedTable)
    {
        unsafe {
            let level_4_table = get_level_4_table(table.p4_frame);
            let mut new_table = MappedTable::new(level_4_table, |frame| phys_to_virt(frame).as_mut_ptr());
            f(&mut new_table);
        }
    }

    pub unsafe fn address() -> PhysAddr {
        Cr3::read().0.start_address()
    }

    pub fn map(&mut self, page: Page, flags: EntryFlags) -> MapperFlush<Size4KiB> {
        let frame = allocate_frames(1).unwrap();
        unsafe {
            self.map_to(page, frame, flags, FRAME_ALLOCATOR.lock().as_mut().unwrap()).unwrap()
        }
    }
}

pub struct InactivePageTable {
    p4_frame: PhysFrame,
}

impl InactivePageTable {
    /// We use the 'map_physical_memory' feature of 'bootloader' crate.
    /// That is we map the whole physical memory in out virtual address space with an offset.
    /// When we want to create a new level 4 page table,we need to do that again.
    /// But as an optimization we link the level 3 page table from current address space to the new address space instead of copying them.
    /// Inspired by this [post](https://os.phil-opp.com/paging-implementation/)
    pub fn new(frame: PhysFrame) -> InactivePageTable {
        let physical_memory_offset = *PHYSICAL_MEMORY_OFFSET.r#try()
            .expect("PHYSICAL_MEMORY_OFFSET not initialized");
        let inactive_table = unsafe { get_level_4_table(frame) };
        let active_table = unsafe { active_level_4_table() };

        let phys_mapped_entry_index = VirtAddr::new(physical_memory_offset).p4_index();
        let old_entry = &active_table[phys_mapped_entry_index];
        let new_entry = &mut inactive_table[phys_mapped_entry_index];
        new_entry.set_addr(old_entry.addr(), old_entry.flags());
        InactivePageTable { p4_frame: frame }
    }

    pub unsafe fn from_address(cr3: u64) -> InactivePageTable {
        InactivePageTable { p4_frame: PhysFrame::containing_address(PhysAddr::new(cr3)) }
    }

    pub unsafe fn address(&self) -> u64 {
        self.p4_frame.start_address().as_u64()
    }
}