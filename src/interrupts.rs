use crate::gdt;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use crate::println;
use crate::print;
use pic8259::ChainedPics;
use spin;
extern crate pc_keyboard;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;// creating two new offsets


#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    KeyBoard,// dont need to specify as 33 it already takes as 33
}
// creating an interrupt for the timer as 32 cause 0 + offset according to PIC 0 is the index of timer

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(
    unsafe{
      ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) // initializing PICs
    }
);

// we use Interrupt table from x86_64 crate we use lazy_static to make sure that we have mutable ref static so that doesn't go out of scope
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);// passing a function pointer named breakpoint (set handler is used inorder to handle the interrupt)
        unsafe{
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }// passing a function pointer named Double fault handler 
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler); // adding a handler to handle the timer interrupt
        idt[InterruptIndex::KeyBoard.as_usize()].set_handler_fn(keyboard_interrupt_handler); // adding a keypress handler to handle the keyboard interrupt
        idt.page_fault.set_handler_fn(page_fault_handler); // adding a fault handler to handle
        idt //return the idt
    
    };

}


pub fn init_idt(){
    IDT.load() //which loads the idt
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame){ 
    println!("BREAKPOINT HANDLER:\n{:#?}", stack_frame); // prints the interrupt if it is available
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) ->  !{
    panic!("DOUBLE FAULT HANDLER:\n {:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: InterruptStackFrame){
    // print!(".");// creating a function that handles timer interrupt
    unsafe{
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8()); // notifying that timer interrupt has ended else it considers PIC's busy
    }
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode){
    use x86_64::registers::control::Cr2;
    use crate::hlt_loop;
    
    println!("EXCEPTION PAGE FAULT");
    println!(" ACCESS ADDRESS : {:?}", Cr2::read());//Cr2 register is Page fault Linear address when page fault occuers it stores program and address
    println!("ERROR CODE: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();// hlt_loop to end the execution

}

extern  "x86-interrupt" fn keyboard_interrupt_handler(stack_frame: InterruptStackFrame){
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;

    lazy_static!{
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));// setting KEYBOARD layout and ScancodeSet to detect input and to handle after ctrl is pressed
    }// handle ctrl is used to map key from ctrl[a-z] we use ignore cause its not needed

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);//initializing port 0x60 so we can access it and its interrupted as data byte
    let scan_code:u8 =unsafe {
        port.read()// getting the value from the port;
    };
    if let Ok(Some(key_event)) = keyboard.add_byte(scan_code){  //translates the SCANCODE into OPTION<key_event> 
        if let Some(key) = keyboard.process_keyevent(key_event){ //key event contains which key caused the event and whether its pressed or released
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),// process_keyevent translates key into character else it matches to a raw key
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    // print!("{}", scan_code);// printing the output
    unsafe{
        PICS.lock().notify_end_of_interrupt(InterruptIndex::KeyBoard.as_u8()); 
    }
}

#[test_case]
fn test_breakpoint_exception(){
    x86_64::instructions::interrupts::int3();
}




