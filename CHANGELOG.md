# 1.0.2
- Change implementation from using `LazyCell::force_mut` to `<LazyCell as DerefMut>::deref_mut` since `force_mut` isn't stabilized until rust 1.94, supporting older rust compilers

# 1.0.1
- Add a changelog in `CHANGELOG.md`
- Make library `#![no_std]`
- Make the lazy
- Change implementation from `UnsafeCell<Option<T>>` to `UnsafeCell<LazyCell<T>>` which for some fixes so that dumballoc-alloced tls:s work through static c libs 

# 1.0.0
- Release dumballoc
