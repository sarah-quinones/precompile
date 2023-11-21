use std::ops::Div;

#[precompile::precompile]
#[precompile_with[std::time::Duration, u32]]
#[cfg_attr(debug_assertions, precompile_with[usize, usize])]
extern "Rust" fn foo<A: Div<B, Output = A>, B>((a, b): (A, B)) -> A {
    a / b
}

#[precompile::precompile]
#[precompile_with(u32)]
pub fn generic_fn<T>() {
    // ...
}
