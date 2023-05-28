#![no_std]// dont link the Rust standard library
#![no_main]// disable all for rustentry points
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]


use::blog_os::println;
use core::panic::PanicInfo;
// use std::alloc::System;
use blog_os::{hlt_loop, memory, allocator};
use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::Page;
use blog_os::memory::BootInfoFrameAllocator;

use x86_64::{VirtAddr, structures::paging::Translate};

extern crate alloc;
use alloc::{boxed::Box, vec::Vec, rc::Rc, vec};


entry_point!(kernel_main);

#[no_mangle] // dont mangle the name of this function or else it will be turned into a random name
// pub extern "C" fn _start(boot_info: &'static BootInfo) -> !{
fn kernel_main(boot_info: &'static BootInfo) -> !{
    // let vga_buffer = 0xb8000 as *mut u8;
    // this function is the entry point, since the linker looks for this function
    // named '_start' by default
    println!("hello world");
    blog_os::init();
    
    use blog_os::memory::translate_addr;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    // new: initialize a mapper
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {BootInfoFrameAllocator::init(&boot_info.memory_map)};

    //mapping to a unused page
    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page,&mut mapper,&mut frame_allocator);


    let page_ptr:*mut u64 = page.start_address().as_mut_ptr();
    unsafe{
            // write the string `New!` to the screen through the new mapping

        page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)
    };

    // let addresses = [0xb8000, 0x201008, 0x0100_0020_1a10, boot_info.physical_memory_offset];

    // for address in addresses {
    //     let virt = VirtAddr::new(address);

    //     //new: use the `mapper.translate_addr` method
    //     let phys = mapper.translate_addr(virt);
    //     //Importing translate translate trait so we can use mapper
    //     println!("{:?} --> {:?}", virt, phys);
    // }

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("initialization failed");
    // let x:Box<u8>= Box::new(41);//creating a new box
    // println!("{:p}", x);//location of box

    // let mut vec = Vec::new();
    // for i in 0..500{
    //     vec.push(i);
    // }
    // println!("vec at {:p}", vec.as_slice());
    
    // let reference_counter = Rc::new(vec![1,2,3,4]);
    // let cloned_reference_counter = reference_counter.clone();
    // println!("cloned Reference count is {:?}", Rc::strong_count(&cloned_reference_counter));
    // core::mem::drop(reference_counter);
    // println!("current cloned Reference count {:?}", Rc::strong_count(&cloned_reference_counter));

    async fn async_number() -> u32 {
        42
    }

    async fn example_task() {
        let number = async_number().await;
        println!("{}", number);
    }



    #[cfg(test)]
    test_main();

    println!("didn't crash");

    hlt_loop();
    
}

// this function is the called on panic on not test
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}
// this function is the called on panic test


// TO RUN cargo build --target thumbv7em-none-eabihf
// cargo run --target x86_64-blog_os.json -- -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin  

//dyn is used to call functions structs or enum that uses the trait

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo)-> ! {
    blog_os::test_panic_handle(info);
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
