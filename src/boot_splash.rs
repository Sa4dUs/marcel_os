use crate::{log::LogType, println, vga_buffer::WRITER};

/// A struct representing the boot screen of the system.
pub struct BootScreen;

impl BootScreen {
    /// Displays the boot screen, which includes an ASCII logo and system information.
    ///
    /// This function clears the screen and prints:
    /// - An ASCII logo from settings.
    /// - The current version of the system.
    /// - Developer information.
    pub fn show() {
        {
            let mut writer = WRITER.lock();
            writer.clear_screen();
        }

        println!("\n{}", crate::settings::ASCII_LOGO);
        println!("Version 0.1.0 - alpha\n");
        println!("Developed by @sa4dus\n");
        println!();
    }

    /// Logs a message with a specific status indicator (e.g., success, info, warning).
    ///
    /// # Arguments
    /// * `status` - The status of the log message, indicating its type (e.g., info, success, etc.).
    /// * `message` - The message to log.
    ///
    /// This function prints the message prefixed with a symbol representing its status:
    /// - `*` for `Info`
    /// - `+` for `Success`
    /// - `x` for `Failed`
    /// - `!` for `Warning`
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
