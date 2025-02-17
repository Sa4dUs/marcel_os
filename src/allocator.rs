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

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

pub const HEAP_START: usize = 0x444444440000;
pub const HEAP_SIZE: usize = 100 * 1024;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    BootScreen::log(LogType::Info, "Initializing heap");

    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    BootScreen::log(LogType::Info, "Allocating frames for heap pages");

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

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should never be called")
    }
}

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
