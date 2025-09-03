#[cfg(feature = "derive")]
pub use alloc_zeroed_macros::AllocZeroed;

/// # Safety
/// All-zero pattern must be a valid value of type.
pub unsafe trait AllocZeroed: Sized {
    fn alloc_zeroed(mem: &mut [u8]) -> Option<&mut Self> {
        use core::mem;

        let size = mem::size_of::<Self>();
        let align = mem::align_of::<Self>();
        let len = mem.len();

        let mem_ptr = mem.as_mut_ptr();
        let offset = mem_ptr.align_offset(align);

        if offset == usize::MAX || size + offset > len {
            return None;
        }

        unsafe {
            let ptr = mem_ptr.add(offset) as *mut Self;
            ptr.write_bytes(0, 1);
            ptr.as_mut()
        }
    }
}

/// # Examples
///
/// ```
/// use alloc_zeroed::{AllocZeroed, alloc_zeroed};
///
/// let value = alloc_zeroed::<u32>().unwrap();
/// assert_eq!(*value, 0);
/// ```
pub fn alloc_zeroed<T: AllocZeroed>() -> Option<Box<T>> {
    use std::alloc::{Layout, alloc_zeroed};

    // Handle zero-sized types without instantiating T
    if std::mem::size_of::<T>() == 0 {
        // For zero-sized types, we can use a dangling pointer
        let dangling_ptr = std::ptr::NonNull::<T>::dangling().as_ptr();
        // SAFETY: For zero-sized types, Box::from_raw with a dangling pointer is safe
        // because zero-sized types don't require actual memory allocation
        return Some(unsafe { Box::from_raw(dangling_ptr) });
    }

    // For non-zero-sized types, use the allocator
    let layout = Layout::new::<T>();

    // SAFETY: This unsafe block is safe because:
    // 1. We've verified that T is not zero-sized
    // 2. We've created a valid Layout for T
    // 3. alloc_zeroed returns null on allocation failure, which we check
    // 4. The returned pointer is properly aligned for T (guaranteed by Layout::new)
    // 5. The memory is zero-initialized, which is valid for T (guaranteed by AllocZeroed trait bound)
    // 6. Box::from_raw will properly manage the memory using the correct Layout
    unsafe {
        let ptr = alloc_zeroed(layout);
        if ptr.is_null() {
            return None;
        }

        let obj_ptr = ptr as *mut T;
        Some(Box::from_raw(obj_ptr))
    }
}

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
