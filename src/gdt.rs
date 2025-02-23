use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

use crate::boot_splash::BootScreen;
use crate::log::LogType;

/// The index of the Double Fault handler in the Interrupt Stack Table (IST).
/// This is used to define the stack for the Double Fault interrupt.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // The stack pointer for the Double Fault interrupt, pointing to the top of the stack.
            let stack_start = VirtAddr::from_ptr(&raw const STACK);

            stack_start + STACK_SIZE
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        // Add a code segment descriptor to the GDT
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        // Add a TSS descriptor to the GDT, linking it with the TSS instance
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

/// A structure to hold the selectors for the code segment and the TSS segment in the GDT.
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// Initializes the Global Descriptor Table (GDT) and the Task State Segment (TSS).
/// This function:
/// - Loads the GDT.
/// - Sets the code segment register (CS).
/// - Loads the TSS to set up the interrupt stack for the system.
pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    // Log the start of GDT initialization.
    BootScreen::log(LogType::Info, "Initializing Global Descriptor Table");
    GDT.0.load();
    BootScreen::log(
        LogType::Success,
        "Global Descriptor Table loaded successfully",
    );

    // Set the code segment and load the TSS for interrupt handling.
    unsafe {
        BootScreen::log(LogType::Info, "Setting code segment register");
        CS::set_reg(GDT.1.code_selector);
        BootScreen::log(LogType::Info, "Initializing Task State Segment");
        load_tss(GDT.1.tss_selector);
        BootScreen::log(LogType::Success, "Task State Segment loaded successfully");
    }
}
