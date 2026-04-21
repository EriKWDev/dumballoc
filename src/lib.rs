#![no_std]

#[macro_export]
/// allocates a given type in a thread-local static variable so that subsequent calls at the same callsite will reuse
/// the same allocation
///
/// use `dumballoc_clear!` to automatically call `dumballoc!` and `.clear()` on the result
macro_rules! dumballoc {
    ($T:ty) => {{
        /*
            NOTE: Calling them 'DUMB' makes it easier to identify these dumballoc threadlocals in readelf debugging and whatnot..
        */
        $crate::dumballoc!(DUMB, $T)
    }};
    ($name:ident, $T:ty) => {{
            thread_local! {
                static $name: ::core::cell::UnsafeCell<::core::cell::LazyCell<$T>> = const { ::core::cell::UnsafeCell::new(::core::cell::LazyCell::new(<$T as Default>::default)) };
            }
            $name.with(|it| {
                let ptr = it.get();
                let ptr = unsafe { &mut *ptr };
                // ::core::cell::LazyCell::force_mut(ptr) // not stable until 1.94
                ::core::ops::DerefMut::deref_mut(ptr)
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
