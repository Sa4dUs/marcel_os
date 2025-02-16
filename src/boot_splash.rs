use crate::{
    log::LogType,
    println,
    vga_buffer::{BUFFER_WIDTH, WRITER},
};

pub struct BootScreen;
impl BootScreen {
    pub fn show() {
        {
            let mut writer = WRITER.lock();
            writer.clear_screen();
        }

        println!("\n{}", crate::settings::ASCII_LOGO);
        println!("Developed by @sa4dus\n");
        println!();
    }

    pub fn log(status: LogType, message: &str) {
        let header: &str = match status {
            LogType::Info => "*",
            LogType::Success => "+",
            LogType::Failed => "x",
            LogType::Warning => "!",
        };

        println!("[{}] {}", header, message);
    }
}
