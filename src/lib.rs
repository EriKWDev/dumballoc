#[macro_export]
/// allocates a given type in a thread-local static variable so that subsequent calls at the same callsite will reuse
/// the same allocation
///
/// use `dumballoc_clear!` to automatically call `dumballoc!` and `.clear()` on the result
macro_rules! dumballoc {
    ($T:ty) => {{
        $crate::dumballoc!(DUMB, $T)
    }};

    ($name:ident, $T:ty) => {{
            thread_local! {
                static $name: std::cell::UnsafeCell<Option<$T>> = const { std::cell::UnsafeCell::new(None) };
            }
            $name.with(|it| unsafe {
                let ptr = it.get();
                (*ptr).get_or_insert_default()
            })
    }};
}

#[macro_export]
/// same as `dumballoc` but also calls '.clear()' on the allocated variable before returning it
macro_rules! dumballoc_clear {
    ($T:ty) => {
        $crate::dumballoc_clear!(DUMB, $T)
    };

    ($name:ident, $T:ty) => {{
        let it = $crate::dumballoc!($name, $T);
        it.clear();
        it
    }};
}
