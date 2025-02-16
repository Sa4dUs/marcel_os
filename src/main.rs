#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![test_runner(marcel_os::test_runner)]
#![feature(custom_test_frameworks)]

extern crate alloc;

use alloc::format;
use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::fmt::write;
use core::panic::PanicInfo;
use marcel_os::boot_splash::BootScreen;
use marcel_os::log::LogType;
use marcel_os::memory::{self, BootInfoFrameAllocator};
use marcel_os::println;
use marcel_os::task::executor::Executor;
use marcel_os::task::keyboard::print_keypresses;
use marcel_os::task::simple_executor::SimpleExecutor;
use marcel_os::task::{keyboard, Task};
use marcel_os::{allocator, boot_splash};
use x86_64::VirtAddr;

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
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

    BootScreen::log(LogType::Success, "Boot sequence finished successfuly");

    BootScreen::show();

    let mut executor = Executor::new();
    executor.run();

    #[cfg(test)]
    test_main();

    marcel_os::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
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
