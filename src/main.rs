#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![test_runner(marcel_os::test_runner)]
#![feature(custom_test_frameworks)]

use core::panic::PanicInfo;
use marcel_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello, World{}", "!");
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    marcel_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
