#![no_std]
#![no_main]

use blog_os::{QemuExitCode, exit_qemu, serial_println, serial_print};
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test case did not panic]");
    exit_qemu(QemuExitCode::Failure);

    loop{};
}

pub fn should_fail() {
    serial_print!("should_panic::should_fail()");
    assert_eq!(0, 1);

}

#[panic_handler]
pub fn _panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);

    loop{};
}