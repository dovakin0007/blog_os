
use crate::gdt;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use crate::println;


// we use Interrupt table from x86_64 crate we use lazy_static to make sure that we have mutable ref static so that doesn't go out of scope
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);// passing a function pointer named breakpoint (set handler is used inorder to handle the interrupt)
        unsafe{
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }// passing a function pointer named Double fault handler 
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


#[test_case]
fn test_breakpoint_exception(){
    x86_64::instructions::interrupts::int3();
}




