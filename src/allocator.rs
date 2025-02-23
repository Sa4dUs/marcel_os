use crate::{
    allocator::fixed_size_block::FixedSizeBlockAllocator, boot_splash::BootScreen, log::LogType,
};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub mod bump;
pub mod fixed_size_block;
pub mod linked_list;

/// The global allocator used by the system.
///
/// This allocator is an instance of `FixedSizeBlockAllocator`, which is locked for safe concurrent access.
#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

/// The start address of the heap in memory.
pub const HEAP_START: usize = 0x444444440000;
/// The size of the heap in bytes.
pub const HEAP_SIZE: usize = 100 * 1024;

/// Initializes the heap by mapping the required memory pages and setting up the allocator.
/// This function maps a range of pages for the heap, allocates frames, and sets up the heap allocator.
///
/// # Arguments
/// * `mapper` - A mutable reference to the `Mapper` for mapping pages.
/// * `frame_allocator` - A mutable reference to the `FrameAllocator` used to allocate physical frames.
///
/// # Returns
/// A `Result` indicating success or failure. Returns `Ok(())` if the heap is successfully initialized,
/// or an error if frame allocation or mapping fails.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    BootScreen::log(LogType::Info, "Initializing heap");

    // Define the page range for the heap.
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    BootScreen::log(LogType::Info, "Allocating frames for heap pages");

    // Allocate and map frames for each page in the heap range.
    for page in page_range {
        let frame = match frame_allocator.allocate_frame() {
            Some(frame) => frame,
            None => {
                BootScreen::log(
                    LogType::Failed,
                    "Frame allocation failed during heap initialization",
                );
                return Err(MapToError::FrameAllocationFailed);
            }
        };

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            // Map the page to the allocated frame.
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    BootScreen::log(LogType::Success, "Heap pages mapped successfully");

    unsafe {
        BootScreen::log(LogType::Info, "Initializing the heap allocator");
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
        BootScreen::log(LogType::Success, "Heap allocator initialized successfully");
    }

    BootScreen::log(LogType::Success, "Heap initialized successfully");

    Ok(())
}

/// A dummy allocator that does not perform any actual allocation or deallocation.
/// This is useful for handling cases where no memory allocation is required or should be allowed.
pub struct Dummy;

/// Implementing `GlobalAlloc` for the `Dummy` allocator.
///
/// This allocator returns `null_mut()` for allocations and panics when attempting to deallocate.
unsafe impl GlobalAlloc for Dummy {
    /// Allocates memory. Since this is a dummy allocator, it always returns `null_mut()`.
    ///
    /// # Arguments
    /// * `_layout` - The memory layout describing the allocation size.
    ///
    /// # Returns
    /// A null pointer (`null_mut()`), indicating that no actual memory is allocated.
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    /// Deallocates memory. This is a no-op that panics if called.
    ///
    /// # Arguments
    /// * `_ptr` - The pointer to the memory to deallocate.
    /// * `_layout` - The layout describing the memory that would be deallocated.
    ///
    /// # Panics
    /// This function always panics because deallocation is not supported in this dummy allocator.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should never be called")
    }
}

/// A wrapper around a `spin::Mutex` to provide safe, locked access to the inner allocator.
/// The `Locked` type ensures that only one thread can access the allocator at a time.
pub struct Locked<A> {
    /// The inner `spin::Mutex` that provides exclusive access to the allocator.
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    /// Creates a new `Locked` instance with the provided inner value.
    ///
    /// # Arguments
    /// * `inner` - The value to be wrapped in the `Locked` mutex.
    ///
    /// # Returns
    /// A `Locked` instance containing the provided value.
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    /// Locks the inner mutex and returns a guard for accessing the wrapped value.
    ///
    /// # Returns
    /// A `MutexGuard` that allows mutable access to the wrapped value.
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

/// Aligns a given address upwards to the nearest multiple of the specified alignment.
/// This function ensures that the address returned is aligned according to the given boundary.
///
/// # Arguments
/// * `addr` - The address to align.
/// * `align` - The alignment boundary to align the address to.
///
/// # Returns
/// The aligned address.
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
