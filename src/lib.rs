#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main="test_main"]
#![feature(abi_x86_interrupt)] 
#![feature(const_mut_refs)]

pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod task;


use core::panic::PanicInfo;
extern crate alloc;

//we declare the exit code as success and failure in the Enum
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u32)]
pub enum QemuExitCode{
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    
    unsafe{
        let mut port = Port::new(0xf4);
        port.write(exit_code as u8)
    }
}
// this function creates a new port at 0xf4 which is the iobase of the isa-debug-exit device
// it writes the passed exit code to the port  which is an unsafe operation
// we make sure that the default exit codes of qemu dont collide with our exit codes
//testable is created to print all 

pub trait Testable {
    fn run(&self) -> ();
}
//implementing Fn trait for printing Ok and calling the function passed in and printing function name


impl<T: Fn()> Testable for T {
    fn run(&self){
        serial_print!("{}.....\t", core::any::type_name::<T>());// core::any::type_name::<T>() return the function in string representation
        self();
        serial_println!("[Ok]");
    }
}

pub fn test_runner(tests :&[&dyn Testable]){
    serial_println!("running {} tests", tests.len());
    for test in tests{
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handle(info: &PanicInfo) -> !{
    serial_println!("[failed]\n");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Failure);

    hlt_loop()
}

use bootloader::entry_point;
use bootloader::BootInfo;

#[cfg(test)]
entry_point!(test_kernel_main);

#[no_mangle]
#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo)->!{
    init();
    test_main();
    
    hlt_loop()

}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo)-> ! {
    test_panic_handle(info);
}

pub fn init(){
    interrupts::init_idt();
    gdt::init();
    unsafe{ interrupts::PICS.lock().initialize() };// initializing interrupt in PICS
    x86_64::instructions::interrupts::enable(); //calling an interrupt

}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();// allows cpu to go into sleep mode until next interrupt arrives
    }
}

