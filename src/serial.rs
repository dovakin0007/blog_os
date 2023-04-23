use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static!{
pub static ref SERIAL1: Mutex<SerialPort> = {
    let mut serial_port = unsafe{ SerialPort::new(0x3F8)};
    serial_port.init();
    Mutex::new(serial_port)
};
}
// same like vga buffer we use Mutex and lazy_static to make static writer instnace 
// like isa-debug-exit UART is programmed for port IO and we are passing the port address as 0x3f8

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments){
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($args:tt)*) => {
        $crate::serial::_print(format_args!($($args)*));
    };
}
/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($args:tt)*) => ($crate::serial_print!(concat!($fmt,"\n"), $($args)*));
}