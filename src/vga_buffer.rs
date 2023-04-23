#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LighGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White =15
}
//This Enum holds all color's which are supported by VGA

use lazy_static::lazy_static;
use spin::Mutex;

//using lazy static to make sure that in Struct non const values are used and mutable values can be accessed and changed
lazy_static!{
    //Mutex is used to for making the program safe and make it accessible for only single thread at a time
    //(note:I am not sure why its used please do help me I am new)
    
    pub static ref WRITER: Mutex<ScreenWriter> = Mutex::new(ScreenWriter {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe {
        &mut *(0xb8000 as *mut Buffer)
          //creating a raw mutable pointer as Buffer and dereferencing it to mutable reference
        }

    }
);
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(forground: Color, background:Color)-> ColorCode {
        ColorCode((background as u8) << 4 | (forground as u8))
    }
}

// ColorCode struct represents the color code in which the foreground and background should be represented. in VGA 
// where we shift background 4 places left(bytes) and the add foreground to the(byte) 

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(C)]
pub struct ScreenChar{
    ascii_char: u8,
    color_code: ColorCode,
    
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer{
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct ScreenWriter{
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}
use volatile::Volatile;
impl ScreenWriter{
    pub fn write_byte(&mut self, byte: u8){
        match byte{
            b'\n' => self.new_line(),
            byte => {
                if self.column_position == BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT-1;
                let col = self.column_position;

                let color_code = self.color_code;

                self.buffer.chars[row][col].write(ScreenChar{
                    ascii_char: byte,
                    color_code,
                });
                self.column_position +=1;
            }
        }
    }

    // Which matches if the char is new line else ascii character will be inserted into the buffer at the row and column position

    fn new_line(&mut self){
        for row in 1..BUFFER_HEIGHT{
            for column in 0..BUFFER_WIDTH{
                let charcters =self.buffer.chars[row][column].read();
                self.buffer.chars[row-1][column].write(charcters);
            }
        }
        self.clear_row(BUFFER_HEIGHT-1);
        self.column_position = 0;
    }
    //we read and move all the columns one row up and call clear_row method

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar{
            ascii_char: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH{
            self.buffer.chars[row][col].write(blank);
        } 
    }
    // this method is used to write all the row with full blank characters 
    
    fn write_string(&mut self, s: &str){
        for byte in s.bytes(){
            match byte {
                //printable asscii or newline
                0x20..=0x7e|b'\n' => self.write_byte(byte),
                //not part of printable asscii
                _ => self.write_byte(0xfe),
            }
        }
    }
    //converts strings into bytes and writes them into the buffer
}

use core::fmt;

impl fmt::Write for ScreenWriter{
    fn write_str(&mut self, s:&str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
//implementing Write Trait for ScreenWriter Struct so we can get write variables in our buffer

#[macro_export] 
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*)=>($crate::print!("{}\n", format_args!($($arg)*)));
}
// implementing macro for println which gets arguments and passes to print fmt::arguments is used here to pass in the string as args
#[macro_export]
macro_rules! print {
    ($($args:tt)*) => {($crate::vga_buffer::_print(format_args!($($args)*)));
      // the print macro takes in arguments and passes into _print function   
    };
}

#[doc(hidden)]

pub fn _print(args: fmt::Arguments){
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
    //now the given arguments are passed to the write_str which implements Writw Trait and which in turn implements buffer
}




// use core::fmt::Write;
// pub fn print_something(){
//     let mut screen_writer = ScreenWriter {
//         column_position:0,
//         color_code:ColorCode::new(Color::Yellow, Color::Black),
//         buffer: unsafe {
//             &mut *(0xb8000 as *mut Buffer) 
//           
//         }
//     };
//     screen_writer.write_byte(b'H');
//     screen_writer.write_string("ello asheriuhu");
//     write!(screen_writer, "the following are numbers are {}, {}", 80, 2.0/5.0).unwrap();
// }