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

        // Handle zero-sized types
        if size == 0 {
            // SAFETY: Zero-sized types don't require actual memory
            let dangling_ptr = std::ptr::NonNull::<Self>::dangling().as_ptr();
            return unsafe { Some(&mut *dangling_ptr) };
        }

        if offset == usize::MAX {
            return None;
        }

        if size > len.saturating_sub(offset) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_allocation() {
        let boxed_int = alloc_zeroed::<u32>().unwrap();
        assert_eq!(*boxed_int, 0);

        let boxed_float = alloc_zeroed::<f64>().unwrap();
        assert_eq!(*boxed_float, 0.0);

        let boxed_bool = alloc_zeroed::<bool>().unwrap();
        assert!(!(*boxed_bool));
    }

    #[test]
    fn test_array_allocation() {
        let boxed_array = alloc_zeroed::<[u32; 10]>().unwrap();
        assert_eq!(*boxed_array, [0; 10]);
    }

    #[test]
    fn test_tuple_allocation() {
        let boxed_tuple = alloc_zeroed::<(u32, u8, bool)>().unwrap();
        assert_eq!(*boxed_tuple, (0, 0, false));
    }

    #[test]
    fn test_zst_allocation() {
        #[derive(Debug, PartialEq)]
        struct Zst;

        unsafe impl AllocZeroed for Zst {}

        let boxed_zst = alloc_zeroed::<Zst>().unwrap();
        assert_eq!(*boxed_zst, Zst);
    }

    #[test]
    fn test_custom_struct_allocation() {
        #[derive(Debug, PartialEq)]
        struct Point {
            x: f64,
            y: f64,
            z: f64,
        }

        unsafe impl AllocZeroed for Point {}

        let boxed_point = alloc_zeroed::<Point>().unwrap();
        assert_eq!(
            *boxed_point,
            Point {
                x: 0.0,
                y: 0.0,
                z: 0.0
            }
        );
    }

    #[test]
    fn test_insufficient_memory() {
        // Test with a buffer that's too small
        let mut small_buffer = [0u8; 4]; // Too small for a u64
        let result = u64::alloc_zeroed(&mut small_buffer);
        assert!(result.is_none());
    }

    #[test]
    fn test_exact_size_buffer() {
        // Test with a buffer that's exactly the right size
        let mut exact_buffer = [0u8; std::mem::size_of::<u32>()];
        let result = u32::alloc_zeroed(&mut exact_buffer);
        assert!(result.is_some());
        assert_eq!(*result.unwrap(), 0);
    }

    #[test]
    fn test_alignment_requirements() {
        // Test with a type that has specific alignment requirements
        #[repr(align(16))]
        struct Aligned(u32);

        unsafe impl AllocZeroed for Aligned {}

        let boxed_aligned = alloc_zeroed::<Aligned>().unwrap();
        assert_eq!(boxed_aligned.0, 0);

        // Check that the pointer is properly aligned
        let ptr = &boxed_aligned as *const _ as *const u8 as usize;
        assert_eq!(ptr % 16, 0);
    }
}
