use crate::boot_splash::BootScreen;
use crate::gdt::DOUBLE_FAULT_IST_INDEX;
use crate::hlt_loop;
use crate::log::LogType;
use crate::println;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

/// The offset for the first PIC (Programmable Interrupt Controller).
/// This is where the interrupts from the first PIC start.
pub const PIC_1_OFFSET: u8 = 32;

/// The offset for the second PIC, which handles interrupts 40-47.
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// A spin-locked instance of the chained PICs with the defined offsets.
/// This is used for managing the interrupts from both PICs.
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// Enum representing the interrupt indices corresponding to the PIC offsets.
/// These are used to handle specific interrupt vectors, such as Timer and Keyboard interrupts.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    /// Converts the interrupt index to a `u8` value.
    fn as_u8(self) -> u8 {
        self as u8
    }

    /// Converts the interrupt index to a `usize` value, used for array indexing.
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // Set the handler for the breakpoint interrupt (INT 3)
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        unsafe {
            // Set the handler for the double fault interrupt and specify the stack index
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        // Set the handlers for page fault, timer, and keyboard interrupts
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

/// Initializes the Interrupt Descriptor Table (IDT) and loads it into the CPU's IDT register.
/// The IDT contains the handlers for various exceptions and interrupts in the system.
pub fn init_idt() {
    BootScreen::log(LogType::Info, "Initializing Interrupt Descriptor Table");
    IDT.load();
    BootScreen::log(
        LogType::Success,
        "Interrupt Descriptor Table loaded successfully",
    );
}

/// Handler for the breakpoint interrupt (INT 3), triggered by the `x86_64::instructions::interrupts::int3()` instruction.
/// This is used for debugging and halting execution at specific points in the code.
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// Handler for the double fault interrupt. This occurs when a fault happens during another interrupt/exception.
/// The handler takes the stack frame and error code (unused in this case) and performs a panic.
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame)
}

/// Handler for the timer interrupt (usually from the Programmable Interval Timer).
/// This is used to manage time-based operations such as scheduling tasks.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        // Notify the PIC that the timer interrupt has been handled.
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

/// Handler for page fault interrupts. This occurs when the processor accesses an invalid memory address.
/// It provides details on the fault, including the error code and the address that caused the fault.
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop(); // Halt the system in case of a page fault.
}

/// Handler for the keyboard interrupt, triggered when a key is pressed.
/// It reads the scancode from the keyboard port and adds it to the keyboard input buffer.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    // Read the scancode from the keyboard's data port (0x60)
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    // Notify the PIC that the keyboard interrupt has been handled.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

/// A test case that triggers a breakpoint exception using the `int3` instruction.
/// This will invoke the breakpoint handler defined above.
#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
