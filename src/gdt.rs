use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0; //we can define any index

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss:TaskStateSegment = TaskStateSegment::new(); //creating tss 
        // we
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK:[u8; STACK_SIZE] = [0; STACK_SIZE];// we use array till we implement proper memory system. Static mut is used if static is
            // used it will only beb able to read the value unable to write

            let stack_start = VirtAddr::from_ptr(unsafe {
                &STACK
            });
            let stack_end = stack_start + STACK_SIZE;

            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt =  GlobalDescriptorTable::new();
        let code_selectors = gdt.add_entry(Descriptor::kernel_code_segment());// gdt is used to switch from kernel space to user space and to load TSS structures
        let tss_selectors = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selectors, tss_selectors } )
    };
}

pub struct Selectors {
    code_selectors: SegmentSelector,
    tss_selectors: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};


    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selectors);//used to set the code segement
        load_tss(GDT.1.tss_selectors);// used to load the tss segement
    }
}