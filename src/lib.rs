#[cfg(feature = "derive")]
pub use alloc_zeroed_macros::AllocZeroed;

/// # Safety
/// All-zero pattern must be a valid value of type.
pub unsafe trait AllocZeroed: Sized {
    fn alloc_zeroed(mem: &mut [u8]) -> Result<&mut Self, AllocError> {
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
            return unsafe { Ok(&mut *dangling_ptr) };
        }

        if offset == usize::MAX {
            return Err(AllocError::AlignmentFailed {
                required_alignment: align,
                address: mem_ptr as usize,
            });
        }

        if size > len.saturating_sub(offset) {
            return Err(AllocError::BufferTooSmall {
                required: size,
                available: len.saturating_sub(offset),
                alignment: align,
            });
        }

        // SAFETY: We've checked that the offset is valid and there's enough space
        let ptr = unsafe { mem_ptr.add(offset) as *mut Self };

        // SAFETY: We've ensured the pointer is properly aligned and there's enough space
        unsafe {
            ptr.write_bytes(0, 1);
            Ok(&mut *ptr)
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
pub fn alloc_zeroed<T: AllocZeroed>() -> Result<Box<T>, AllocError> {
    use std::alloc::{Layout, alloc_zeroed};

    let layout = Layout::new::<T>();
    if std::mem::size_of::<T>() == 0 {
        // For zero-sized types, we can use a dangling pointer
        let dangling_ptr = std::ptr::NonNull::<T>::dangling().as_ptr();
        // SAFETY: For zero-sized types, Box::from_raw with a dangling pointer is safe
        // because zero-sized types don't require actual memory allocation
        return Ok(unsafe { Box::from_raw(dangling_ptr) });
    }

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
            return Err(AllocError::OutOfMemory {
                required: layout.size(),
                alignment: layout.align(),
            });
        }

        let obj_ptr = ptr as *mut T;
        Ok(Box::from_raw(obj_ptr))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    /// Not enough space in the provided buffer (for trait method)
    BufferTooSmall {
        required: usize,
        available: usize,
        alignment: usize,
    },
    /// The global allocator is out of memory (for free function)
    OutOfMemory { required: usize, alignment: usize },
    /// Unable to align the pointer in the provided buffer
    AlignmentFailed {
        required_alignment: usize,
        address: usize,
    },
    /// The type has an invalid size or alignment
    InvalidLayout { size: usize, alignment: usize },
}

impl std::fmt::Display for AllocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllocError::BufferTooSmall {
                required,
                available,
                alignment,
            } => write!(
                f,
                "required {} bytes (with {} alignment) but only {} bytes available in buffer",
                required, alignment, available
            ),
            AllocError::OutOfMemory {
                required,
                alignment,
            } => write!(
                f,
                "out of memory: required {} bytes with {} alignment",
                required, alignment
            ),
            AllocError::AlignmentFailed {
                required_alignment,
                address,
            } => write!(
                f,
                "could not align address {} to required alignment {}",
                address, required_alignment
            ),
            AllocError::InvalidLayout { size, alignment } => {
                write!(f, "invalid layout: size={}, alignment={}", size, alignment)
            }
        }
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
        assert!(matches!(
            result,
            Err(AllocError::BufferTooSmall {
                required: 8,
                available: _,
                alignment: _
            })
        ));
    }

    #[test]
    fn test_exact_size_buffer() {
        // Test with a buffer that's exactly the right size
        let mut exact_buffer = [0u8; std::mem::size_of::<u32>()];
        let result = u32::alloc_zeroed(&mut exact_buffer);
        assert!(result.is_ok());
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
        let ptr = &*boxed_aligned as *const _ as *const u8 as usize;
        assert_eq!(ptr % 16, 0);
    }

    #[test]
    fn test_alloc_error_display() {
        // Test BufferTooSmall
        assert_eq!(
            AllocError::BufferTooSmall {
                required: 100,
                available: 50,
                alignment: 8
            }
            .to_string(),
            "required 100 bytes (with 8 alignment) but only 50 bytes available in buffer"
        );

        // Test OutOfMemory
        assert_eq!(
            AllocError::OutOfMemory {
                required: 1024,
                alignment: 16
            }
            .to_string(),
            "out of memory: required 1024 bytes with 16 alignment"
        );

        // Test AlignmentFailed
        assert_eq!(
            AllocError::AlignmentFailed {
                required_alignment: 16,
                address: 0x1001
            }
            .to_string(),
            "could not align address 4097 to required alignment 16"
        );

        // Test InvalidLayout
        assert_eq!(
            AllocError::InvalidLayout {
                size: 0,
                alignment: 16
            }
            .to_string(),
            "invalid layout: size=0, alignment=16"
        );
    }

    #[test]
    fn test_alloc_error_debug() {
        // Test BufferTooSmall
        assert!(
            format!(
                "{:?}",
                AllocError::BufferTooSmall {
                    required: 100,
                    available: 50,
                    alignment: 8
                }
            )
            .contains("BufferTooSmall")
        );

        // Test OutOfMemory
        assert!(
            format!(
                "{:?}",
                AllocError::OutOfMemory {
                    required: 1024,
                    alignment: 16
                }
            )
            .contains("OutOfMemory")
        );

        // Test AlignmentFailed
        assert!(
            format!(
                "{:?}",
                AllocError::AlignmentFailed {
                    required_alignment: 16,
                    address: 0x1001
                }
            )
            .contains("AlignmentFailed")
        );

        // Test InvalidLayout
        assert!(
            format!(
                "{:?}",
                AllocError::InvalidLayout {
                    size: 0,
                    alignment: 16
                }
            )
            .contains("InvalidLayout")
        );
    }

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn test_alloc_error_clone_and_partial_eq() {
        // Test that errors can be cloned and compared
        let err1 = AllocError::BufferTooSmall {
            required: 100,
            available: 50,
            alignment: 8,
        };
        let err2 = err1.clone();
        assert_eq!(err1, err2);

        let err3 = AllocError::OutOfMemory {
            required: 1024,
            alignment: 16,
        };
        assert_ne!(err1, err3);

        // Test that different instances with same values are equal
        let err4 = AllocError::BufferTooSmall {
            required: 100,
            available: 50,
            alignment: 8,
        };
        assert_eq!(err1, err4);

        // Test that different instances with different values are not equal
        let err5 = AllocError::BufferTooSmall {
            required: 200, // Different required size
            available: 50,
            alignment: 8,
        };
        assert_ne!(err1, err5);
    }
}
