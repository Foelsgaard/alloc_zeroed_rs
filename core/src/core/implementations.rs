use core::mem::MaybeUninit;

use crate::AllocZeroed;

// Implement AllocZeroed for primitive types
unsafe impl AllocZeroed for u8 {}
unsafe impl AllocZeroed for u16 {}
unsafe impl AllocZeroed for u32 {}
unsafe impl AllocZeroed for u64 {}
unsafe impl AllocZeroed for usize {}
unsafe impl AllocZeroed for i8 {}
unsafe impl AllocZeroed for i16 {}
unsafe impl AllocZeroed for i32 {}
unsafe impl AllocZeroed for i64 {}
unsafe impl AllocZeroed for isize {}
unsafe impl AllocZeroed for bool {}
unsafe impl AllocZeroed for f32 {}
unsafe impl AllocZeroed for f64 {}

// Implement for arrays of AllocZeroed types
unsafe impl<T: AllocZeroed, const N: usize> AllocZeroed for [T; N] {}

// Implement for tuples of AllocZeroed types (up to some reasonable size)
macro_rules! impl_tuple {
    ($($T:ident),+) => {
        unsafe impl<$($T: AllocZeroed),+> AllocZeroed for ($($T,)+) {}
    }
}

impl_tuple!(A);
impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);
impl_tuple!(A, B, C, D, E);
impl_tuple!(A, B, C, D, E, F);
impl_tuple!(A, B, C, D, E, F, G);
impl_tuple!(A, B, C, D, E, F, G, H);

// SAFETY: MaybeUninit<T> can safely contain any bit pattern, including all zeros.
// The default implementation of alloc_zeroed will zero the memory, which is always
// safe for MaybeUninit<T> regardless of T.
unsafe impl<T> AllocZeroed for MaybeUninit<T> {}
