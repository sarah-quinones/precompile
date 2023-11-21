#![allow(incomplete_features)]
#![feature(specialization)]
#![no_std]

#[doc(hidden)]
pub trait Impl: Sized {
    const FN_PTR: *const ();
}

#[doc(hidden)]
pub trait PickOr<Generic: Impl> {
    fn pick_or(self, generic: Generic) -> *const ();
}
impl<T, Generic: Impl> PickOr<Generic> for T {
    #[inline(always)]
    default fn pick_or(self, _generic: Generic) -> *const () {
        Generic::FN_PTR
    }
}
impl<T: Impl, Generic: Impl> PickOr<Generic> for T {
    #[inline(always)]
    fn pick_or(self, _generic: Generic) -> *const () {
        T::FN_PTR
    }
}

#[inline(always)]
#[doc(hidden)]
pub fn pick<Generic: Impl, Spec>(generic: Generic, spec: Spec) -> *const () {
    <Spec as PickOr<Generic>>::pick_or(spec, generic)
}

pub use precompile_macro::precompile;
