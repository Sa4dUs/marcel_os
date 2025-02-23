use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

/// A low-level function for printing formatted text to the serial port.
///
/// This function writes the formatted text to the serial port, ensuring that
/// interrupts are disabled during the operation to avoid data corruption.
///
/// # Arguments
/// * `args` - The formatted string arguments to be printed.
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // Disable interrupts to safely write to the serial port without interruptions.
    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// A macro for printing to the serial port without a newline.
///
/// This macro accepts a format string with arguments and prints the resulting
/// string to the serial port.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// A macro for printing to the serial port with a newline.
///
/// This macro is similar to `serial_print!`, but appends a newline to the
/// printed string.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
