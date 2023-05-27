use core::{alloc::{Layout, GlobalAlloc}, ptr::{self, NonNull}, mem};
use super::Locked;



// the block size to use 
//the sizes mut each be power of two because they also used as the block alignment
const BLOCK_SIZE: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

struct ListNode{
    next:  Option<&'static mut ListNode>
}

pub struct  FixedSizeBlockAllocator{
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZE.len()],// field of head pointers one for each blocksize
    fallback_allocator: linked_list_allocator::Heap,// fallback allocator as linked list allocator if allocation is larger than field in array
}

impl  FixedSizeBlockAllocator {
    pub const fn new() -> Self{

        //creates a new empty block allocator
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator { 
            list_heads: [EMPTY; BLOCK_SIZE.len()], 
            fallback_allocator: linked_list_allocator::Heap::empty() }
    }

 

    //initialize the allocator with the given heap bounds

    //the function is unsafe because the caller must gurantee that the given heap bounds are valid
    // and the heap is unused this method must be called once
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize){
        self.fallback_allocator.init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self, layout:Layout)->*mut u8{
        //returns ptr on success 
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

fn list_index(layout: &Layout) -> Option<usize>{
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZE.iter().position(|&s| s>= required_block_size)
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8{
        let mut allocator = self.lock();// gets mutable reference 
        match list_index(&layout){// calculate the appropriate block size
            Some(index) => {
                match allocator.list_heads[index].take(){// if list index is Some we remove the first element using take
                    Some(node)=>{// if the list is not empty we get the linked list node and take it
                        allocator.list_heads[index]= node.next.take();
                        node as *mut ListNode as *mut u8//and return it as a u8 pointer address
                    }
                    None =>{
                        //no block exists in list => allocate new block
                        let block_size = BLOCK_SIZE[index];// if the block is empty we create a new block
                        //only works if all block sizes in power of 2
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout)// calling the fallback allocator to create a new linked list allocator
                    }
            }
        }
        None => allocator.fallback_alloc(layout),// if nothing sets  or if its None then we fallback calling linked list
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8,layout: Layout) {
        let mut allocator = self.lock();// gets the allocator refernce
        match list_index(&layout){
            Some(index) => {// if it exists then we need to deallocate it and free from the list and it to the freed memory
                let new_node = ListNode{
                    next: allocator.list_heads[index].take()//we create a ListNode pointing to current list head then we write a new list node to that position
                };

                //verify that the block has size to store the list node
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZE[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZE[index]);
                
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr)
                
            }
            None => {// if index is None deallocate from fall back allocator
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }
}