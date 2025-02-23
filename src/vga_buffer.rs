use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        cursor_position: (0, 0),
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// Enumeration of the available colors for text and background.
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
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Represents a color code, combining a foreground and background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Creates a new color code with the given foreground and background colors.
    ///
    /// # Arguments
    /// * `foreground` - The color to use for the text.
    /// * `background` - The color to use for the background.
    ///
    /// # Returns
    /// A new `ColorCode` representing the combination of the foreground and background.
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode(((background as u8) << 4) | (foreground as u8))
    }
}

/// Represents a single character on the screen, including its ASCII value and color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// The height of the screen buffer in rows.
pub const BUFFER_HEIGHT: usize = 25;

/// The width of the screen buffer in columns.
pub const BUFFER_WIDTH: usize = 80;

/// Represents the VGA buffer, which stores all characters displayed on the screen.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// The writer that handles output to the VGA buffer, including cursor management and text color.
pub struct Writer {
    cursor_position: (usize, usize),
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Writes a single byte to the screen, advancing the cursor position.
    ///
    /// If the byte is a newline (`\n`), a new line is started. Otherwise, the character
    /// is written at the current cursor position.
    ///
    /// # Arguments
    /// * `byte` - The byte to write, typically an ASCII character.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.cursor_position.1 >= BUFFER_WIDTH {
                    self.new_line();
                }

                let (row, col) = self.cursor_position;
                let color_code = self.color_code;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.cursor_position.1 += 1;
            }
        }

        self.update_cursor();
    }

    /// Writes a string to the screen, character by character.
    ///
    /// # Arguments
    /// * `s` - The string to write to the screen.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Clears the entire screen and resets the cursor position to the top-left corner.
    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.cursor_position = (0, 0);
        self.update_cursor();
    }

    /// Starts a new line by either moving the cursor to the next row or scrolling the screen.
    fn new_line(&mut self) {
        if self.cursor_position.0 >= BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
            self.cursor_position.0 = BUFFER_HEIGHT - 1;
        } else {
            self.cursor_position.0 += 1;
        }

        self.cursor_position.1 = 0;
    }

    /// Clears a specific row by writing spaces to every column in the row.
    ///
    /// # Arguments
    /// * `row` - The row to clear.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// Moves the cursor back one character without wrapping.
    pub fn move_cursor_back(&mut self) {
        if self.cursor_position.1 > 0 {
            self.cursor_position.1 -= 1;
            self.update_cursor();
        }
    }

    /// Updates the hardware cursor to the current cursor position.
    pub fn update_cursor(&mut self) {
        let (row, col) = self.cursor_position;
        let pos = row * BUFFER_WIDTH + col;

        unsafe {
            use x86_64::instructions::port::Port;
            let mut port_cmd = Port::new(0x3D4);
            let mut port_data = Port::new(0x3D5);

            port_cmd.write(0x0F_u8);
            port_data.write((pos & 0xFF) as u8);
            port_cmd.write(0x0E_u8);
            port_data.write(((pos >> 8) & 0xFF) as u8);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Prints formatted text to the screen, using the `Writer`'s `write_fmt` method.
///
/// This macro is a wrapper around the `write_fmt` function to enable formatted printing.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Prints formatted text to the screen with a newline at the end.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Internal function to print text to the screen.
///
/// This function is used by the `print!` and `println!` macros to write formatted
/// text to the VGA buffer.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

/// A simple test case for printing a single line.
#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

/// A test case for printing multiple lines.
#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

/// A test case that verifies the output of `println!` by comparing the printed string
/// with the contents of the VGA buffer.
#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
