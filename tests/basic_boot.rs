#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(marcel_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use marcel_os::{exit_qemu, println, QemuExitCode};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    marcel_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println! output");
}
