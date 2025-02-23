#![no_std]
#![cfg_attr(test, no_main)]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![feature(abi_x86_interrupt)]

extern crate alloc;
use core::panic::PanicInfo;

use boot_splash::BootScreen;
#[cfg(test)]
use bootloader::{entry_point, BootInfo};
use log::LogType;

pub mod allocator;
pub mod boot_splash;
pub mod cli;
pub mod gdt;
pub mod interrupts;
pub mod log;
pub mod memory;
pub mod serial;
pub mod settings;
pub mod task;
pub mod vga_buffer;

/// Initializes various kernel components, including:
/// - The Global Descriptor Table (GDT)
/// - The Interrupt Descriptor Table (IDT)
/// - Programmable Interrupt Controllers (PICs)
/// - Enables CPU interrupts
///
/// This function is called at the start of the kernel's execution.
pub fn init() {
    // Initialize GDT and IDT
    gdt::init();
    interrupts::init_idt();

    // Initialize PICs
    unsafe {
        BootScreen::log(
            LogType::Info,
            "Initializing Programmable Interrupt Controllers",
        );
        interrupts::PICS.lock().initialize();
        BootScreen::log(
            LogType::Success,
            "Programmable Interrupt Controllers initialized successfully",
        );
    }

    // Enable CPU interrupts
    BootScreen::log(LogType::Info, "Enabling interrupts");
    x86_64::instructions::interrupts::enable();
    BootScreen::log(LogType::Success, "Interrupts enabled");
}

/// Trait for marking types that can be tested in the kernel test suite.
pub trait Testable {
    /// Runs the test function.
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    /// Executes the test and prints the result.
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

/// The test runner that executes a list of tests and exits with a result code.
/// It uses `serial_print!` and `serial_println!` to print test results to the serial console.
/// After all tests, the QEMU exit code is sent to indicate success or failure.
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

/// Handles kernel panics during tests by printing the error message to the serial console
/// and exiting the QEMU emulator with a failure code.
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

/// Enum representing the QEMU exit codes used to signal the result of test execution.
/// `Success` signals that tests passed, while `Failed` indicates that at least one test failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Exits the QEMU virtual machine with a specific exit code.
/// This is typically used after tests to indicate success or failure.
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4); // QEMU exit port
        port.write(exit_code as u32); // Write the exit code to signal QEMU
    }
}

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for the kernel test suite when the `test` configuration is enabled.
/// This function initializes the kernel and runs the tests.
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
/// Panic handler for the test environment. This ensures that kernel panics during tests
/// are properly handled, including printing the panic information and exiting QEMU with the failure code.
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

/// A simple infinite loop that halts the CPU. This is typically used when the kernel
/// encounters an error or after the test suite completes.
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt(); // Halt the CPU to prevent it from running wild.
    }
}
