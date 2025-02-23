use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};

/// Represents a node in the free list for memory allocation.
struct ListNode {
    /// The size of the free memory block.
    size: usize,
    /// Pointer to the next free block in the list.
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    /// Creates a new list node with the given size.
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    /// Returns the starting address of the node.
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    /// Returns the ending address of the node.
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

/// A simple linked list-based memory allocator.
pub struct LinkedListAllocator {
    /// Head node of the free list.
    head: ListNode,
}

impl LinkedListAllocator {
    /// Creates a new empty allocator.
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// Initializes the allocator with a given heap range.
    ///
    /// # Safety
    /// This function must be called only once and with a valid heap memory range.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    /// Adds a new free memory region to the allocator.
    ///
    /// # Safety
    /// The given memory region must be valid and not already allocated.
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());

        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr);
    }

    /// Searches for a suitable free memory region that satisfies the requested size and alignment.
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(region, size, align) {
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                current = current.next.as_mut().unwrap();
            }
        }
        None
    }

    /// Attempts to allocate memory from a given region, ensuring proper alignment.
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            return Err(());
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            return Err(());
        }

        Ok(alloc_start)
    }

    /// Adjusts the given memory layout to fit the allocator's constraints.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

impl Default for LinkedListAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Implements the global allocator trait for `LinkedListAllocator`.
unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    /// Allocates a memory block with the given layout.
    ///
    /// # Safety
    /// The caller must ensure that the requested allocation is valid and does not cause undefined behavior.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    /// Deallocates a previously allocated memory block.
    ///
    /// # Safety
    /// The caller must ensure that the pointer and layout are valid and that the memory block was allocated by this allocator.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size);
    }
}
