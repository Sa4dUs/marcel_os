#![no_std]
#![no_main]
mod memory;
mod vga_buffer;

use core::panic::PanicInfo;

use vga_buffer::print_something;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello, World{}", "!");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
