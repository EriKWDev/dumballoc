pub const NUM_OUTER_ITERATIONS: usize = 1000;
pub const NUM_INNER_ITERATIONS: usize = 100;

pub static TOTAL_ALLOCATION: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
pub static TOTAL_DEALLOCATION: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

pub struct CountingAllocator(std::alloc::System);

#[global_allocator]
pub static GLOBAL: CountingAllocator = CountingAllocator(std::alloc::System);

unsafe impl core::alloc::GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        TOTAL_ALLOCATION.fetch_add(layout.size(), std::sync::atomic::Ordering::Relaxed);
        unsafe { self.0.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        TOTAL_DEALLOCATION.fetch_add(layout.size(), std::sync::atomic::Ordering::Relaxed);
        unsafe { self.0.dealloc(ptr, layout) }
    }
}
