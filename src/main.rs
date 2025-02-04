#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![test_runner(marcel_os::test_runner)]
#![feature(custom_test_frameworks)]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use marcel_os::println;

entry_point!(kernel_main);
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    println!("Hello, World!");
    marcel_os::init();

    #[cfg(test)]
    test_main();

    println!("It did not crash");
    marcel_os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    marcel_os::hlt_loop();
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
