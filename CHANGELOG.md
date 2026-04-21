# 1.0.1
- Add a changelog in `CHANGELOG.md`
- Make library `#![no_std]`
- Make the lazy
- Change implementation from `UnsafeCell<Option<T>>` to `UnsafeCell<LazyCell<T>>` which for some fixes so that dumballoc-alloced tls:s work through static c libs 

# 1.0.0
- Release dumballoc
