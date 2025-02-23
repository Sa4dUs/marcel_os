#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![test_runner(marcel_os::test_runner)]
#![feature(custom_test_frameworks)]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use marcel_os::allocator;
use marcel_os::boot_splash::BootScreen;
use marcel_os::cli::{cli, init_cli};
use marcel_os::log::LogType;
use marcel_os::memory::{self, BootInfoFrameAllocator};
use marcel_os::task::executor::Executor;
use marcel_os::task::Task;
use x86_64::VirtAddr;

entry_point!(kmain);

/// Kernel main function, responsible for initializing the system and entering the main loop.
///
/// # Arguments
/// * `boot_info` - A reference to bootloader-provided system information.
fn kmain(boot_info: &'static BootInfo) -> ! {
    BootScreen::log(LogType::Info, "Initializing boot sequence");
    marcel_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    BootScreen::log(LogType::Info, "Initializing memory mapper");
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    BootScreen::log(LogType::Success, "Memory mapper initialized successfully");

    BootScreen::log(LogType::Info, "Initializing frame allocator");
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    BootScreen::log(LogType::Success, "Frame allocator initialized successfully");

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    BootScreen::log(LogType::Info, "Initializing Command Line Interface");
    init_cli();
    BootScreen::log(LogType::Success, "Command Line Interface initialized");

    BootScreen::log(LogType::Success, "Boot sequence finished successfully");
    BootScreen::show();

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(cli()));
    executor.run();

    #[allow(unreachable_code)]
    marcel_os::hlt_loop();
}

/// Panic handler for non-test environments.
///
/// # Arguments
/// * `info` - Panic information.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use marcel_os::println;

    println!("{}", info);
    marcel_os::hlt_loop();
}

/// Panic handler for test environments.
///
/// # Arguments
/// * `info` - Panic information.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    marcel_os::test_panic_handler(info)
}

/// A trivial test case to verify basic functionality.
#[allow(clippy::eq_op)]
#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
