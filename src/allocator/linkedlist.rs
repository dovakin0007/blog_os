// use core::mem;
// use super::Locked;
// use alloc::alloc::{GlobalAlloc, Layout};
// use core::ptr;

// use linked_list_allocator::align_up;


// pub struct ListNode{
//     size: usize,
//     next: Option<&'static mut ListNode>, // &'static describes an owned object behind pointer itsn a box without a deconstructor that fress object at end of the scope
// }

// impl ListNode{
//     const fn new(size: usize)->Self{
//         ListNode { size, next: None }
//     }   

//     fn start_addr(&self)->usize{
//         self as *const Self as usize
//     }

//     fn end_addr(&self)-> usize{
//         self.start_addr() + self.size
//     }
// }

// pub struct  LinkListAllocator{
//     head: ListNode, //creating a head pointer
// }

// impl LinkListAllocator{

//     //creates a new linked list allocator
//     pub const fn new()-> Self{
//         LinkListAllocator { head: ListNode::new(0) }//const is used so it will be used to initialize the allocator
//     }

//     pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize){
//         self.add_free_region(heap_start, heap_size);// it is used to add free region  in memory
//     }

// // this method takes the address and memory region as argument and adds it to the front of the list
//     unsafe fn add_free_region(&mut self, addr: usize, size: usize){
//         // ensure that the freed region is capable of holding ListNode
//         assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
//         assert!(size >= mem::size_of::<ListNode>());
//         //ensuring that size and alignment are storing a list node then it create another list node and inserts it into existing list
//         //create a new list node and append it to the start of the list node
//         let mut node = ListNode::new(size);
//         node.next = self.head.next.take();
//         let node_ptr = addr as *mut ListNode;
//         node_ptr.write(node);
//         self.head.next = Some(&mut *node_ptr);
//     }

//     //looks for free region with the given size and alignments and removes it from the list
//     //Returns a tuple of the list node and the start location
//     fn find_region(&mut self, size: usize, align: usize) ->
//     Option<(&'static mut ListNode, usize)> {

//         //reference to current list node updates on each iteration
//         let mut current = &mut self.head;
//         // Look for a large enough memory region in linked list
//         while let Some(ref mut region) = current.next {
//             if let Ok(alloc_start) = Self::alloc_from_region(&region, size , align){
//                  //region suitable for allocation -> remove from list node
//                  let next = region.next.take();
//                  let ret = Some((current.next.take().unwrap(), alloc_start));
//                 current.next = next;
//                 return  ret;
//             }else{
//                 //region not suitable for allocation -> move to next node
//                 current.next.as_mut().unwrap();
//             }
            
//         }
//         None
//     }
//     //Try to use the given region for allocation with the given size and alignment
//     //return Start address on success
//     fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> 
//     Result<usize, ()>{
//         let alloc_start = align_up(region.start_addr(), align);
//         let alloc_end =  alloc_start.checked_add(size).ok_or(())?;
    
//         //region to small
//         if alloc_end > region.end_addr(){
//             return Err(());
//         }
//         let excess_size = region.end_addr() - alloc_end;
//         if excess_size > 0 && excess_size < mem::size_of::<ListNode>(){
//             //rest of the region is to small to hold list node (required because allocation splits the region into used part and free part)
//             return Err(());
//         }
//         Ok(alloc_start)
//     }
// //adjust the given layout so that the resulting allocated memory 
// //region is also capable of storing list node
// // returns the adjusted size and aligment as (size, align) tuple.
//     fn size_align(layout: Layout) -> (usize, usize) {  
//         //to increase the aligment to the aligment of list nodes
//         let layout = layout.align_to(mem::align_of::<ListNode>()).expect("adjustment alignment failed").pad_to_align();
//         // pad to align ensures that next memeory block is aligned to store List node
//         let size = layout.size().max(mem::size_of::<ListNode>());
//         // it uses max method to enforce a minimum allocation size of  mem::size_of::<ListNode>
        
//         (size, layout.align())
//     }


// }

// unsafe impl GlobalAlloc for Locked<LinkListAllocator>{
//     unsafe fn alloc(&self, layout: Layout) -> *mut u8{
//         //perform layout adjustments

//         let (size, align) = LinkListAllocator::size_align(layout);
//         let mut allocator = self.lock();

//         if let Some((region, alloc_start)) = allocator.find_region(size, align){
//             let alloc_end = alloc_start.checked_add(size).expect("overflow");
//             let excess_size  = region.end_addr() - alloc_end; //gets the excess size
//             if excess_size > 0{
//                 allocator.add_free_region(alloc_end, excess_size);// adds excess region to free space
//             } 
//             alloc_start as *mut u8   
//         }else{
//             ptr::null_mut()// return error
//         }
//     }

//      unsafe fn dealloc(&self,ptr: *mut u8, layout: Layout) {
//         //perform layout adjustments
//         let (size, _) = LinkListAllocator::size_align(layout);

//         self.lock().add_free_region(ptr as usize, size)
//     }

 
// }

use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    /// Creates an empty LinkedListAllocator.
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// This function is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the heap is unused. This method must be
    /// called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    /// Adds the given memory region to the front of the list.
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // ensure that the freed region is capable of holding ListNode
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());

        // create a new list node and append it at the start of the list
        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr)
    }

    /// Looks for a free region with the given size and alignment and removes
    /// it from the list.
    ///
    /// Returns a tuple of the list node and the start address of the allocation.
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        // reference to current list node, updated for each iteration
        let mut current = &mut self.head;
        // look for a large enough memory region in linked list
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                // region suitable for allocation -> remove node from list
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                // region not suitable -> continue with next region
                current = current.next.as_mut().unwrap();
            }
        }

        // no suitable region found
        None
    }

    /// Try to use the given region for an allocation with given size and alignment.
    ///
    /// Returns the allocation start address on success.
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            // region too small
            return Err(());
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            // rest of region too small to hold a ListNode (required because the
            // allocation splits the region in a used and a free part)
            return Err(());
        }

        // region suitable for allocation
        Ok(alloc_start)
    }

    /// Adjust the given layout so that the resulting allocated memory
    /// region is also capable of storing a `ListNode`.
    ///
    /// Returns the adjusted size and alignment as a (size, align) tuple.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // perform layout adjustments
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // perform layout adjustments
        let (size, _) = LinkedListAllocator::size_align(layout);

        self.lock().add_free_region(ptr as usize, size)
    }
}
