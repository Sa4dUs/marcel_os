use crate::{println, vga_buffer::WRITER};
use alloc::string::String;
use conquer_once::spin::OnceCell;
use core::sync::atomic::AtomicBool;
use spin::Mutex;
use x86_64::instructions::port::Port;

pub static COMMAND_BUFFER: OnceCell<Mutex<String>> = OnceCell::uninit();
pub static COMMAND_READY: AtomicBool = AtomicBool::new(false);

pub fn init_cli() {
    COMMAND_BUFFER
        .try_init_once(|| Mutex::new(String::new()))
        .expect("Command buffer should only be initialized once");
}

pub async fn cli() {
    use crate::print;
    use crate::println;
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

        while let Some(scancode) = scancodes.next().await {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => match character {
                            '\n' => {
                                print!("\n");
                                parse(&buffer);
                                buffer.clear();
                                break;
                            }
                            '\x08' | '\x7F' => {
                                if !buffer.is_empty() {
                                    buffer.pop();
                                    unsafe {
                                        let mut writer = WRITER.lock();
                                        writer.move_cursor_back();
                                        writer.write_byte(b' ');
                                        writer.move_cursor_back();
                                    }
                                }
                            }
                            _ => {
                                buffer.push(character);
                                print!("{}", character);
                            }
                        },
                        DecodedKey::RawKey(_) => {}
                    }
                }
            }
        }
    }
}

fn parse(buffer: &str) {
    match buffer.trim() {
        "help" => {
            println!("Available commands:");
            println!("  help     - Show this help menu");
            println!("  hello    - Print 'Hello, World!'");
            println!("  clear    - Clear the creen");
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
        "" => {}
        _ => {
            println!("Unknown command. Type 'help' for a list of commands.");
        }
    }
}
