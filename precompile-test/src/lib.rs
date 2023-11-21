#[precompile::precompile]
#[precompile_with(u32)]
pub fn generic_fn<T: From<u32>>(slice: &mut [T]) {
    for x in slice {
        *x = 1312u32.into();
    }
}
