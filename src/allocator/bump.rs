use core::ptr;

use alloc::alloc::{ GlobalAlloc, Layout};

pub struct BumpAllocator{
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self{
        BumpAllocator { heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0 }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_end: usize) {
        self.heap_start = heap_start;// keeps track of upper bound 
        self.heap_end = heap_start + heap_end; //keeps track of lower bound 
        self.next = heap_start;// next is used to point to next location so that we dont point to same location twice
    }
    
}

//lock is used to prevent data race which will cause a deadlock
unsafe impl GlobalAlloc for Locked<BumpAllocator>{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8{
        // TODO alignment and bounds checking
        let mut bump = self.lock(); //get a mutable reference
        let alloc_start = align_up(bump.next, layout.align());// getting the start of the frame allocator and assigning it to the variab;e
        let alloc_end = match alloc_start.checked_add(layout.size()) {// getting the end of the allocator till where the data is assigned or returns a null pointer
            //checked add prevents overflow when a large allocation happens else we return a null pointer
            Some(end) => end,
            None => return ptr::null_mut(),
        };
        if alloc_end>bump.heap_end{
            ptr::null_mut()// returns null pointer if the allocated pointer goes out of the heap  out of memory
        }else {
            bump.next = alloc_end;//set the next value of bump to end of the allocator 
            bump.allocations += 1;// adds 1 at the end of the allocator 
            alloc_start as *mut u8// returns the location of the memory allocated
        }

    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout){
        let mut bump =self.lock();

        bump.allocations -=1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}

//Align the given address `addr` upwards to alignment `align`.
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

//creating own wrapper 
pub struct Locked<A>{
    inner: spin::Mutex<A>,//spin mutex is a much better verison of default mutex in rust
}

impl <A> Locked<A> {
    pub const fn new(inner: A)-> Locked<A> {
        Locked { inner: spin::Mutex::new(inner) }// creates a new mutex
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock() //locks the value till it gets out of scope
    }
}