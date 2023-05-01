#![no_std]// dont link the Rust standard library
#![no_main]// disable all for rustentry points
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]


use::blog_os::println;
use core::panic::PanicInfo;


#[no_mangle] // dont mangle the name of this function or else it will be turned into a random name
pub extern "C" fn _start() -> !{
    
    // let vga_buffer = 0xb8000 as *mut u8;
    // this function is the entry point, since the linker looks for this function
    // named '_start' by default
    println!("hello world");

    blog_os::init();
    //x86_64::instructions::interrupts::int3();// calling interrupt int 3

    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 4;
    // };

    fn stack_overflow(){
        stack_overflow();
    }//invoking stack overflow 

    stack_overflow();

    #[cfg(test)]
    test_main();

    println!("didn't crash");
    loop {}
}

// this function is the called on panic on not test
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
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
