use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub static mut TSS: TaskStateSegment = TaskStateSegment::new();

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        unsafe {
            let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
            (
                gdt,
                Selectors {
                    code_selector,
                    tss_selector,
                },
            )
        }
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

/// about [pti](https://en.wikipedia.org/wiki/Kernel_page-table_isolation)
#[cfg(feature = "pti")]
pub unsafe fn set_tss_stack(stack: usize) {
    unimplemented!()
//    use arch::x86_64::pti::{PTI_CPU_STACK, PTI_CONTEXT_STACK};
//    TSS.rsp[0] = (PTI_CPU_STACK.as_ptr() as usize + PTI_CPU_STACK.len()) as u64;
//    PTI_CONTEXT_STACK = stack;
}

#[cfg(not(feature = "pti"))]
pub unsafe fn set_tss_stack(stack: usize) {
    TSS.privilege_stack_table[0] = VirtAddr::new(stack as u64);
}