#![no_std]// dont link the Rust standard library
#![no_main]// disable all for rustentry points
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod serial;
mod vga_buffer;

use core::panic::PanicInfo;
// use vga_buffer::WRITER;
#[no_mangle] // dont mangle the name of this function or else it will be turned into a random name
pub extern "C" fn _start() -> !{
    
    // let vga_buffer = 0xb8000 as *mut u8;
    // this function is the entry point, since the linker looks for this function
    // named '_start' by default
    println!("hello world");
    

    #[cfg(test)]
    test_main();

    exit_qemu(QemuExitCode::Success);
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
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failure);
    loop {}
}

// TO RUN cargo build --target thumbv7em-none-eabihf
// cargo run --target x86_64-blog_os.json -- -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin  

//dyn is used to call functions structs or enum that uses the trait
#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}
//we declare the exit code as success and failure in the Enum
pub fn exit_qemu(exit_code: QemuExitCode){
    use x86_64::instructions::port::Port;

    unsafe{
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
    //x86_64 is used to access and write the codes to ports 0xf4
}
// this function creates a new port at 0xf4 which is the iobase of the isa-debug-exit device
// it writes the passed exit code to the port  which is an unsafe operation
// we make sure that the default exit codes of qemu dont collide with our exit codes
//testable is created to print all 
pub trait Testable {
    fn run(&self)  -> ();
}

//implementing Fn trait for printing Ok and calling the function passed in and printing function name

impl<T:Fn()> Testable for T{
    fn run(&self) {
        serial_println!("{}..\t", core::any::type_name::<T>());// core::any::type_name::<T>() return the function in string representation
        self();// calls the function
        serial_println!("[ok]");
    }
 }

// impl<T> Testable for T 
// where T:Fn(){
    
// }