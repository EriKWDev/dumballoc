# Dumballoc
A thread-safe reusable allocation declared inline without need for setup

The proposed allocation strategy here is not generally applicable and your application's constraints might be different
to ours.

Please see the `History` and `Solution` sections below to determine whether this is for you

## Usage

```rust
fn function_that_gets_called_a_lot_in_a_hot_loop_but_needs_allocations_and_i_am_too_lazy_to_rewrite_callsite_to_pre_allocate() {
    let stack = dumballoc_clear!(Vec<usize>);
    let map = dumballoc_clear!(HashMap<usize, usize>);

    stack.push(0);

    while let Some(job) = stack.pop() {
        if job < 3 {
            jobs.push(job + 1);
            jobs.push(job + 10);
            map.insert(job, job + 10);
        }
    }
}
```

The call to `dumballoc_clear!` will create a thread-local variable of the given type and call `.clear()` on it meaning that
subsequent calls to the function will re-use any previous `dumballoc` allocations improving performance without the callsite
needing to know

## History

During the development of Swap (`https://idno.se/swap`), we found ourselves at a point where our gamelogic code
required some graph traversal algorithms, temporary acceleration structures and lists and whatnot all the time
but it had very high performance requirements as we needed to be able to evaluate many different and future game
states every frame potentially depending on player input

A common pattern would be something like
```rust
fn some_gamelogic_function() {
    let mut todo = vec![];
    let mut map = HashMap::default();

    while let Some(job) = todo.pop() {
       // find entity for job and construct the map
    }

    // act on the map
}
```

When calling `some_gamleogic_function`, it will allocate and deallocate a vec and map every single time which really is not ideal.
Any memory allocation (unless the global allocator is overriden -- will get back to that later), is a potential system call and a
potential stall


What we started doing was after profiling, for the hottest functions in the codebase, we had to rewrite them such that the memory
allocation instead is up to the caller rather than the callee.

```rust
fn some_gamelogic_function(todo_buf: &mut Vec<Job>), map_buf: &mut HashMap<Key, Value> {
    todo_buf.clear();
    map_buf.clear();

    let todo = todo_buf;
    let map = map_buf;

    while let Some(job) = todo.pop() {
       // find entity for job and construct the map
    }

    // act on the map
}
```

This meant that when we had a hot loop and were calling `some_gamelogic_function` a lot, we could re-use the outer allocation
for every invocation which drastically improved performance

```rust
let mut todo_buf = vec![];
let mut map_buf = HashMap::default();
for entity in entities {
    some_gamelogic_function(&mut todo_buf, &mut map_buf);
}
```

This approach is ok, and I generally find it to be a good way of writing functions in general. However, it does impose extra
work on the function author to also need to care about allocations even before they know whether the allocations it will be
making will actually become a bottleneck in the real application.

Furthermore, having several layers of functions all needing to take in their allocated buffers as parameters, quickly becomes
messy with lots of function arguments all being mutable vecs, maps, vecdeques etc, obfustcating the actually important parameters
to the function

So, we wanted a solution that would allow us to re-use allocations for any given algorithm *without* needing to rewrite any function
that calls the algorithm

This meant that the solution needed to be
 - minimally intrusive to code already written
 - minimally or not at all intrusive to any code calling the algorithms
 - preferably simple

## The Solution
The solution we came up with was 11 lines of code that we named `dumballoc!`

```rust
macro_rules! dumballoc {
    ($T:ty) => {{
            thread_local! {
                static __ALLOCATION: std::cell::UnsafeCell<Option<$T>> = const { std::cell::UnsafeCell::new(None) };
            }
            __ALLOCATION.with(|it| unsafe {
                let ptr = it.get();
                (*ptr).get_or_insert_default()
            })
    }};
}
```

and a utility macro called `dumballoc_clear!`

```rust
macro_rules! dumballoc_clear {
    ($T:ty) => {{
        let it = $crate::dumballoc!($T);
        it.clear();
        it
    }};
}
```

(that is the whole library btw)

By allocating a thread-local variable, we know that any function using `dumballoc!` can safely be called from any thread
since every thread will have their own allocations, and by calling `dumballoc_clear!` we can ensure that the re-used allocation
is cleared before the function starts using it.

This we found to be an elegant solution as a rewrite of an old function was now trivial.

```rust
let mut stack = vec![];
// becomes..
let stack = dumballoc_clear!(Vec<usize>);
```

and no other code needs to be touched. If that stack was previously allocated in a hot loop, it will now automatically re-use its buffer.

We had special usecases and constraints (that don't fit everyone or everywhere!), but I wanted to share this as I think this
might be a useful idea even if a bit contrarian in the rust community.

## Alternatives
Of course, there are other ways to solve the problems presented here, and they all have different tradeoffs. Being aware of multiple solutions with different trade-offs is what
allows you to make an informed decision rather than just blindly doing what everyone else does.

Using dumballoc means that the worst-case calling of the functions utilizing it will eventually allocate a buffer of the worst-case size and it will never be de-allocated.
For us, this was a totally fine trade-off, but beware that the total memory usage now becomes the sum of all the worst-cases requirements

It further ***requires special care with recursion*** as each recursive call will utilize the exact same allocation and clear it thus violating safety guarantees. We had no recursive
gamelogic with a perf issue, so `dumballoc` isn't used there.

If this is an issue for you, and it should be in many cases, you should most likely not be using `dumballoc!` there and rather look in to alternatives

I am working on a library to make the below more ergonomic and it will probably be our approach in future projects, but dumballoc was a sweet experiment
that solved our problems very simply

### Passing around a temporary allocator
You can use something like a arena allocator or bumpallocator and pass it around as a argument

This will of course require you to rewrite all your functions to take this argument, but if done from the start as a convention it might be
a feasible strategy

```rust
fn my_hot_function(alloc: &Allocator) {
    let stack = alloc.vec();
    let map = alloc.hashmap();
    /// ...
}
```

### Temporarily overwriting the global allocator
This approach gets me excited!

We already have an implicit global allocator accessible everywhere, so why not adapt it temporarily to better suit our needs?

```rust

let mut state = /* .. */;

loop {
  let mut result = Vec::with_capacity(100);

  scope_with_my_fast_allocator(|| {
    for i in 0..100 {
         let r = call_my_or_thirdparty_stupid_function_that_is_ignorant_to_temporary_allocations_being_a_problem();
         result.push(r);
    }
  });

  reset_my_fast_allocator();
}
```

It is completely transparent to any code you are calling that does allocations, and it can potentially result in huge performance gains.

Of course, you will now become in-charge of making sure that the owned allocations of any returned structures have the correct lifetimes for your allocators

> With great power comes great responsibility

### Using the unstable allocator API
If you enjoy type-spaghetti, then of course you can make every single structure and function in your hot code be generic over an allocator
which seems to be the way rust wants us to do it in the future.

Sounds fun to write and read all those extra brackets

# License
Copyright 2025 Erik W. Gren

This code is licensed under the MIT license. See `LICENSE`
