use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{registers::control::Cr3, PhysAddr, VirtAddr};

use crate::boot_splash::BootScreen;
use crate::log::LogType;

/// Initializes the page table using the physical memory offset.
///
/// This function sets up an `OffsetPageTable` using the Level 4 page table provided
/// by the CPU's current page table register (`Cr3`), and then returns an `OffsetPageTable`
/// that will handle the translation between virtual and physical addresses.
///
/// # Arguments
/// * `physical_memory_offset` - The offset that separates the kernel's virtual address space
///   from the physical memory space.
///
/// # Safety
/// This function is unsafe because it manipulates raw pointers directly.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    BootScreen::log(LogType::Info, "Initializing page table");

    // Retrieve the active Level 4 page table (PML4) from Cr3.
    let level_4_table = active_level_4_table(physical_memory_offset);

    BootScreen::log(LogType::Success, "Level 4 page table loaded successfully");

    // Create an OffsetPageTable from the active Level 4 table.
    let offset_page_table = OffsetPageTable::new(level_4_table, physical_memory_offset);

    BootScreen::log(LogType::Success, "Page table initialized successfully");

    offset_page_table
}

/// Retrieves the currently active Level 4 page table from the CPU's page table register (Cr3).
///
/// This function reads the `Cr3` control register to obtain the physical address of the
/// current Level 4 page table, and maps it into the kernel's virtual address space.
///
/// # Arguments
/// * `physical_memory_offset` - The base address offset to convert the physical address to a
///   virtual address.
///
/// # Safety
/// This function is unsafe because it manipulates raw pointers and relies on the layout of
/// the CPU's internal registers.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();

    // Cast the physical address to a pointer and return the pointer to the Level 4 page table.
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// A frame allocator that doesn't allocate any frames. Used as a placeholder.
///
/// This struct does not implement actual memory allocation and simply returns `None` for every
/// frame allocation request.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    /// Always returns `None` as this allocator does not provide memory frames.
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

/// A frame allocator that uses the bootloader's memory map to allocate usable memory frames.
///
/// This allocator is initialized with the memory map provided by the bootloader and iterates
/// over the usable memory regions to allocate physical memory frames. It only returns frames
/// from memory regions marked as "Usable."
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Initializes the `BootInfoFrameAllocator` from the bootloader-provided memory map.
    ///
    /// This function sets up the allocator to iterate over the usable regions in the memory map.
    ///
    /// # Arguments
    /// * `memory_map` - The memory map provided by the bootloader, detailing the memory regions.
    ///
    /// # Safety
    /// This function is unsafe because it relies on the structure and validity of the memory map.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over usable memory frames.
    ///
    /// This function filters the memory map for usable regions and yields `PhysFrame` objects
    /// corresponding to the frames in those regions.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096)); // 4096-byte frame size.
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

/// Allocates memory frames from the usable regions of physical memory.
///
/// This implementation of `FrameAllocator` uses the bootloader's memory map to allocate usable
/// memory frames for the kernel.
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// Sets a block of memory to a specific value.
///
/// This function writes the `value` byte to `num` consecutive bytes starting from `ptr`.
///
/// # Arguments
/// * `ptr` - A pointer to the start of the memory block to be filled.
/// * `value` - The byte value to fill the memory block with.
/// * `num` - The number of bytes to fill.
///
/// # Returns
/// The original pointer `ptr`.
///
/// # Safety
/// This function is unsafe because it operates directly on raw pointers.
#[no_mangle]
pub extern "C" fn memset(ptr: *mut u8, value: u8, num: usize) -> *mut u8 {
    unsafe {
        let mut p = ptr;
        for _ in 0..num {
            *p = value;
            p = p.add(1);
        }
    }

    ptr
}

/// Copies a block of memory from one location to another.
///
/// This function copies `num` bytes from the source pointer `src` to the destination pointer `dest`.
///
/// # Arguments
/// * `dest` - A pointer to the destination memory block.
/// * `src` - A pointer to the source memory block.
/// * `num` - The number of bytes to copy.
///
/// # Returns
/// The original destination pointer `dest`.
///
/// # Safety
/// This function is unsafe because it operates directly on raw pointers.
#[no_mangle]
pub extern "C" fn memcpy(dest: *mut u8, src: *const u8, num: usize) -> *mut u8 {
    unsafe {
        let mut dest_ptr = dest;
        let mut src_ptr = src;

        for _ in 0..num {
            *dest_ptr = *src_ptr;
            dest_ptr = dest_ptr.add(1);
            src_ptr = src_ptr.add(1);
        }
    }

    dest
}

/// Compares two blocks of memory byte by byte.
///
/// This function compares the first `num` bytes of the memory blocks pointed to by `ptr1` and
/// `ptr2`. If the memory blocks are identical, it returns 0. Otherwise, it returns a positive or
/// negative value based on the first differing byte.
///
/// # Arguments
/// * `ptr1` - A pointer to the first memory block.
/// * `ptr2` - A pointer to the second memory block.
/// * `num` - The number of bytes to compare.
///
/// # Returns
/// A value indicating the result of the comparison: 0 if the blocks are equal, or the difference
/// between the first differing byte.
///
/// # Safety
/// This function is unsafe because it operates directly on raw pointers.
#[no_mangle]
pub unsafe extern "C" fn memcmp(ptr1: *const u8, ptr2: *const u8, num: usize) -> i32 {
    unsafe {
        for i in 0..num {
            let byte1 = *ptr1.add(i);
            let byte2 = *ptr2.add(i);
            if byte1 != byte2 {
                return (byte1 as i32) - (byte2 as i32);
            }
        }
    }
    0
}

/// Moves a block of memory from one location to another, handling overlapping regions.
///
/// This function safely moves `num` bytes from the source pointer `src` to the destination pointer
/// `dest`, taking care to handle cases where the memory regions may overlap.
///
/// # Arguments
/// * `dest` - A pointer to the destination memory block.
/// * `src` - A pointer to the source memory block.
/// * `num` - The number of bytes to move.
///
/// # Returns
/// The original destination pointer `dest`.
///
/// # Safety
/// This function is unsafe because it operates directly on raw pointers.
#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, num: usize) -> *mut u8 {
    unsafe {
        if (dest as *const u8) < src {
            let mut dest_ptr = dest;
            let mut src_ptr = src;
            for _ in 0..num {
                *dest_ptr = *src_ptr;
                dest_ptr = dest_ptr.add(1);
                src_ptr = src_ptr.add(1);
            }
        } else {
            let mut dest_ptr = dest.add(num);
            let mut src_ptr = src.add(num);
            for _ in 0..num {
                dest_ptr = dest_ptr.sub(1);
                src_ptr = src_ptr.sub(1);
                *dest_ptr = *src_ptr;
            }
        }
    }

    dest
}
