use core::mem;

pub use x86_64::structures::paging::{
    mapper::MapperFlush,
    Size4KiB,
};

use super::table::ActivePageTable;

/// To allow for combining multiple flushes into one, we have a way of flushing
/// the active table, which can consume `MapperFlush` structs
#[must_use = "The page table must be flushed, or the changes unsafely ignored"]
pub struct MapperFlushAll(bool);

impl MapperFlushAll {
    /// Create a new promise to flush all mappings
    pub fn new() -> MapperFlushAll {
        MapperFlushAll(false)
    }

    /// Consume a single page flush
    pub fn consume(&mut self, flush: MapperFlush<Size4KiB>) {
        self.0 = true;
        mem::forget(flush);
    }

    /// Flush the active page table
    pub fn flush(self, table: &mut ActivePageTable) {
        if self.0 {
            table.flush_all();
        }
        mem::forget(self);
    }

    /// Ignore the flush. This is unsafe, and a reason should be provided for use
    pub unsafe fn ignore(self) {
        mem::forget(self);
    }
}

/// A flush cannot be dropped, it must be consumed
impl Drop for MapperFlushAll {
    fn drop(&mut self) {
        panic!("Mapper flush all was not utilized");
    }
}