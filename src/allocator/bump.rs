use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

/// A simple bump allocator that allocates memory in a linear fashion.
/// Memory is allocated by incrementing a pointer, and deallocated in bulk.
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Creates a new, uninitialized bump allocator.
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initializes the bump allocator with a given heap start address and size.
    ///
    /// # Safety
    /// This function must only be called once and requires a valid memory region.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start.saturating_add(heap_size);
        self.next = heap_start;
    }
}

impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    /// Allocates a memory block with the specified layout.
    ///
    /// # Safety
    /// This function must be used in a single-threaded environment or protected by a lock.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            ptr::null_mut()
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    /// Deallocates a memory block. Since this is a bump allocator, deallocation
    /// is only meaningful when all allocations are freed at once.
    ///
    /// # Safety
    /// The provided pointer must be valid and previously allocated.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
