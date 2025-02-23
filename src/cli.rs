use crate::{println, vga_buffer::WRITER};
use alloc::string::String;
use conquer_once::spin::OnceCell;
use core::sync::atomic::AtomicBool;
use spin::Mutex;
use x86_64::instructions::port::Port;

/// A static once-initialized buffer for storing the inputted commands.
pub static COMMAND_BUFFER: OnceCell<Mutex<String>> = OnceCell::uninit();

/// A flag indicating if a command has been entered and is ready for processing.
pub static COMMAND_READY: AtomicBool = AtomicBool::new(false);

/// Initializes the command-line interface (CLI) system by setting up the command buffer.
pub fn init_cli() {
    COMMAND_BUFFER
        .try_init_once(|| Mutex::new(String::new()))
        .expect("Command buffer should only be initialized once");
}

/// The asynchronous CLI handler that listens for keyboard input and processes commands.
///
/// This function:
/// - Waits for keypresses from the user via the `ScancodeStream`.
/// - Handles the user input, including backspace and newlines.
/// - Parses the entered command and calls the respective handler.
///
/// The CLI runs in a loop, continually waiting for and processing commands until a command is processed.
pub async fn cli() {
    use crate::print;
    use crate::task::keyboard::ScancodeStream;
    use futures_util::stream::StreamExt;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    let mut buffer = String::new();

    loop {
        print!("> ");

        // Read the scancodes and process the keys.
        while let Some(scancode) = scancodes.next().await {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => match character {
                            // On Enter, process the command and clear the buffer.
                            '\n' => {
                                print!("\n");
                                parse(&buffer);
                                buffer.clear();
                                break;
                            }
                            // Handle backspace.
                            '\x08' | '\x7F' => {
                                if !buffer.is_empty() {
                                    buffer.pop();

                                    let mut writer = WRITER.lock();
                                    writer.move_cursor_back();
                                    writer.write_byte(b' ');
                                    writer.move_cursor_back();
                                }
                            }
                            // Add characters to the buffer.
                            _ => {
                                buffer.push(character);
                                print!("{}", character);
                            }
                        },
                        // Ignore raw keys (non-character keys).
                        DecodedKey::RawKey(_) => {}
                    }
                }
            }
        }
    }
}

/// Parses the entered command and performs the associated action.
///
/// # Arguments
/// * `buffer` - The string buffer containing the user's command.
///
/// This function checks the command and calls the appropriate function:
/// - `help` shows the available commands.
/// - `hello` prints "Hello, World!".
/// - `clear` clears the screen.
/// - `shutdown` shuts down the system.
fn parse(buffer: &str) {
    match buffer.trim() {
        "help" => {
            println!("Available commands:");
            println!("  help     - Show this help menu");
            println!("  hello    - Print 'Hello, World!'");
            println!("  clear    - Clear the screen");
            println!("  shutdown - Power off the system");
        }
        "hello" => {
            println!("Hello, World!");
        }
        "clear" => {
            let mut writer = WRITER.lock();
            writer.clear_screen();
        }
        "shutdown" => {
            println!("Shutting down...");
            unsafe {
                let mut port = Port::new(0x604);
                port.write(0x2000u16);
            }
        }
        // Handle empty input (do nothing).
        "" => {}
        _ => {
            println!("Unknown command. Type 'help' for a list of commands.");
        }
    }
}
