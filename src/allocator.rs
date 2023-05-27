use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, mapper::MapToError
    },
    VirtAddr,
};
use linked_list_allocator::LockedHeap;
use bump::BumpAllocator;
use linkedlist::LinkedListAllocator;
use self::{bump::Locked, fixed_size_block::FixedSizeBlockAllocator};


pub mod fixed_size_block;
pub mod bump;
pub mod linkedlist;


pub const HEAP_START:usize = 0x_4444_4444_0000;
pub const HEAP_SIZE:usize = 100 * 1024;
pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout, )->*mut u8{
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout:Layout){
        panic!("dealloc should never been called");
    }
}

// #[global_allocator]
// static ALLOCATOR: Locked<BumpAllocator>= Locked::new(BumpAllocator::new());

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(
    FixedSizeBlockAllocator::new());

//function takes in arguments that implement mapper trait and frame allocator trait as parameters
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,// default size of 4kb
)-> Result<(), MapToError<Size4KiB>>{
    //creating a new page range
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64); // creating start of the page
        let heap_end = heap_start + HEAP_SIZE - 1u64; //creating end of page 
        let heap_start_page = Page::containing_address(heap_start); //gets the start of the page range
        let heap_end_page = Page::containing_address(heap_end);//gets the end of the page range
        Page::range_inclusive(heap_start_page, heap_end_page) //allocating the page range
    };
    //mapping the pages 
    for page in page_range{
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;// this method returns none and returns \
        //MapToError::FrameAllocationFailed and question mark operator returns early in case of error
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;//to indicate whether the page is writable and present
        unsafe{
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();//create mapping on success and returns mapper flush else forwards the error
        };
        unsafe{
            ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
        }
    
    }
    Ok(())
}

fn align_up(addr: usize, align: usize) -> usize{
    // let remainder = addr % align;
    // if remainder == 0 { // if remainder is 0 its already aligned
    //     addr
    // }else{
    //     addr - remainder + align // else align it by subracting remainder
    // }
    //requires that the align to be powers of two
    (addr + align - 1) & !(align - 1)  
    // let align be 0b000100000 align - 1 be 0b000111111 by creating bit wise not we get 0b…111111111100000
    // performing bitwise AND on (align - 1) we align the address downwards
    //we want to align upwards so we do addr + align - 1 before AND  so already aligned values remain same the ones not are rounded to arbitary boundary
}