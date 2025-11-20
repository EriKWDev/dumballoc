mod common;
use common::*;

fn stupid_function() {
    let ptr = unsafe {
        std::alloc::alloc(core::alloc::Layout::array::<usize>(NUM_INNER_ITERATIONS).unwrap())
    };
    let list = unsafe { Vec::<usize>::from_raw_parts(ptr as _, 0, NUM_INNER_ITERATIONS) };
    let mut list = std::hint::black_box(list);
    for i in 0..NUM_INNER_ITERATIONS {
        list.push(i);
    }
}

#[test]
fn test_default_behaviour_does_many_allocations() {
    TOTAL_ALLOCATION.store(0, std::sync::atomic::Ordering::Relaxed);
    TOTAL_DEALLOCATION.store(0, std::sync::atomic::Ordering::Relaxed);
    for _ in 0..NUM_OUTER_ITERATIONS {
        stupid_function();
    }
    let tot_alloc = TOTAL_ALLOCATION.load(std::sync::atomic::Ordering::Relaxed);
    let tot_dealloc = TOTAL_DEALLOCATION.load(std::sync::atomic::Ordering::Relaxed);

    assert_eq!(
        tot_alloc,
        NUM_OUTER_ITERATIONS * NUM_INNER_ITERATIONS * size_of::<usize>()
    );
    assert_eq!(
        tot_dealloc,
        NUM_OUTER_ITERATIONS * NUM_INNER_ITERATIONS * size_of::<usize>()
    );
}
