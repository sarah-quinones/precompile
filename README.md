# precompile

`precompile` is a nightly-only crate that uses `#![feature(specialization)]`
for precompiling specific monomorphizations of a generic function.
This can provide a benefit for generic functions that are expected to be used
with a limited set of types from multiple downstream crates.

For example:

```rust
// crate: A
pub fn generic_fn<T>() {
    // ...
}

// crate B
A::generic_fn::<u32>();

// crate C
A::generic_fn::<u32>();
```
This code will usually compile `A::generic_fn::<u32>` twice.

Whereas if we precompile `generic_fn`:
```rust
// crate: A
#[precompile::precompile]
#[precompile_with(u32)]
pub fn generic_fn<T>() {
    // ...
}
```

Then no matter how many crates use `A::generic_fn::<u32>`, it will only be
compiled once, when the crate `A` is built.

This means a possibly larger upfront compile-time cost for building `A`, in exchange for
a cheaper monomorphization cost when used downstream.
