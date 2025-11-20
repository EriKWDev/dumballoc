mod common;
use common::*;

fn stupid_function_with_dumballoc() {
    let list = dumballoc::dumballoc_clear!(DUMB, Vec<usize>);
    let list = std::hint::black_box(list);
    list.reserve_exact(NUM_INNER_ITERATIONS);
    for i in 0..NUM_INNER_ITERATIONS {
        list.push(i);
    }
}

#[test]
fn test_dumballoc_doing_less_than_default_allocations() {
    TOTAL_ALLOCATION.store(0, std::sync::atomic::Ordering::Relaxed);
    TOTAL_DEALLOCATION.store(0, std::sync::atomic::Ordering::Relaxed);
    for _ in 0..NUM_OUTER_ITERATIONS {
        stupid_function_with_dumballoc();
    }
    let tot_alloc = TOTAL_ALLOCATION.load(std::sync::atomic::Ordering::Relaxed);
    let tot_dealloc = TOTAL_DEALLOCATION.load(std::sync::atomic::Ordering::Relaxed);

    assert_eq!(tot_alloc, NUM_INNER_ITERATIONS * size_of::<usize>());
    assert_eq!(tot_dealloc, 0);
}
