use x86_64::structures::paging::PageTable;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::structures::paging::frame;
use x86_64::{VirtAddr, PhysAddr};
use x86_64::{
    structures::paging::{Page, PhysFrame, Mapper, Size4KiB, FrameAllocator}
};
use bootloader::bootinfo::MemoryMap;
use bootloader::bootinfo::MemoryRegionType;

pub struct BootInfoFrameAllocator{
    // a frame allocator returns usable frames from the bootloader memory
    memory_map: &'static MemoryMap,
    next: usize,
}
 impl BootInfoFrameAllocator {
    // this function is unsafe cause the user callwer has to guarantee that memory map is valid
    //main requirment is to return all memory maps that are usable in it which are unused
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self{
        BootInfoFrameAllocator { memory_map , next: 0 }
    }

    pub fn usable(&self) -> impl Iterator<Item = PhysFrame>{
        //creating an iterator of memory_map
        let regions = self.memory_map.iter();
        //filter out regions without USABLE Flag
        let usable_regions = regions.filter(|r|{
            r.region_type == MemoryRegionType::Usable
        });
        //mapping each address to address range
        let addr_range = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        //transform to an interator of frame start range
        let frame_addresses = addr_range.flat_map(|m| m.step_by(4096));
        //create physframe type from start address
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB>for BootInfoFrameAllocator {
    fn allocate_frame(&mut self)-> Option<PhysFrame>{
        let frame =self.usable().nth(self.next);
        self.next +=1;
        frame
    }
}


/// Creates an example mapping for the given page to frame `0xb8000`.
// function expects page that should be mapped and a reference to the offset page table and instance of FrameAllocator trait
pub fn create_example_mapping(page: Page, mapper: &mut OffsetPageTable, frame_allocator: &mut impl FrameAllocator<Size4KiB>){
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE; // we use present flag and writeable flag to make the mapped entry writable

    let map_to_result = unsafe {
         // FIXME: this is not safe, we do it only for testing
         // this function is unsafe becuase it will causing undefined behavior 
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to_failed").flush();// map to might fail so we use expect to handle  the result on success returns a mapper flush
}



// returns an empty frame on call
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
      None
  }      
}

//returning a mutable reference to Page Table level 4

//This function is unsafe because the caller must guarantee that the
//complete physical memory is mapped to virtual memory at the passed
// `physical_memory_offset`. Also, this function must be only called once
// to avoid aliasing `&mut` references (which is undefined behavior).

unsafe fn active_level_4_table (physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (l4_page_table_frame, _) = Cr3::read();//reading a l4 page table

    let phys = l4_page_table_frame.start_address(); //get the the start address of the page table
    let virt = physical_memory_offset + phys.as_u64(); //getting the virtual address of the page table we add physical memory offset to l4 table address

    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();// converting into mutable raw pointer

    &mut *page_table_ptr // returning the page table as mutable raw pointer with a static lifetime
}

// this functions returns Offset of the page table
pub unsafe fn init(physical_memory_offset: VirtAddr)->  OffsetPageTable<'static>{
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset) // returns a new offset page table with static lifetime

}

//Translates the mapped Virtual Address into a physical address or return None if address is not mapped
//the function is unsafe beacause the caller has to guarrantee that the mapped Virtual Address exists in physical address at the passed offset
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr>{
    translate_addr_inner(addr, physical_memory_offset)
}

//Private function called by translate_addr

//the function is safe to the limit of scope of unsafe because rust treats the whole block as unsafe code
// this function is only reachable through the unsafe module
fn translate_addr_inner(addr: VirtAddr, physical_memory: VirtAddr) -> Option<PhysAddr>{

    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _)=  Cr3::read(); //gets the start location of l4 table from Cr3 register

    let table_indexes = [addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index(), ]; //contains a list of table addresses
    let mut frame = level_4_table_frame; 

    for &index in &table_indexes{ //iterates through the table indexes


         // converting the index into page table reference

        let virt = physical_memory + frame.start_address().as_u64();//adding the virtual address start and physical address offset
        let table_ptr: *const PageTable = virt.as_ptr();//converting the virtual address to physical address by adding the pointer to virtual address
        let table = unsafe {&*table_ptr};// gettinng the value present in the table
         // converting the index into page table reference

        let entry = &table[index];
        // read the page table entry and update the frame
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };

    }
    //calculating the physical address by adding the page offset 
    Some(frame.start_address() + u64::from(addr.page_offset()))
}