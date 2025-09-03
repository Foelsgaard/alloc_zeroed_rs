extern crate std;

mod error;

use crate::{AllocError, AllocErrorKind, AllocZeroed, alloc_err};
use std::boxed::Box;

pub trait AllocZeroedBoxed: crate::AllocZeroed {
    /// Allocates and zero-initializes an instance of `Self` on the heap.
    ///
    /// This method uses the global allocator to allocate memory for `Self` on the heap,
    /// ensuring proper alignment and zero-initializing the allocated memory. The returned
    /// `Box` will properly manage the memory and call the destructor when dropped.
    ///
    /// # Returns
    ///
    /// * `Ok(Box<Self>)` - A box containing the zero-initialized object if allocation succeeds.
    /// * `Err(AllocError)` - An error describing why allocation failed (out of memory
    ///                       or invalid layout).
    ///
    /// # Errors
    ///
    /// Returns `AllocError` in the following cases:
    /// * `AllocError::OutOfMemory` - The system allocator cannot fulfill the allocation request
    /// * `AllocError::InvalidLayout` - The type has an invalid size or alignment combination
    ///
    /// # Safety
    ///
    /// This method relies on the safety guarantees of `AllocZeroed`, requiring that an
    /// all-zero bit pattern is a valid representation for the type `Self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use alloc_zeroed::{AllocZeroed, AllocZeroedBoxed};
    ///
    /// #[derive(AllocZeroed)]
    /// struct Point {
    ///     x: f64,
    ///     y: f64,
    /// }
    ///
    /// let point = Point::alloc_zeroed_boxed().unwrap();
    /// assert_eq!(point.x, 0.0);
    /// assert_eq!(point.y, 0.0);
    /// ```
    ///
    /// # Zero-Sized Types
    ///
    /// For zero-sized types (ZSTs), this method always succeeds and returns a box containing
    /// a dangling pointer, as ZSTs don't require actual memory allocation.
    ///
    /// # Notes
    ///
    /// This method requires the `std` feature to be enabled, as it uses the global allocator
    /// and `Box` type from the standard library.
    fn alloc_zeroed_boxed() -> Result<Box<Self>, AllocError> {
        use AllocErrorKind::*;
        use std::alloc::{Layout, alloc_zeroed};

        let layout = Layout::new::<Self>();
        if std::mem::size_of::<Self>() == 0 {
            // For zero-sized types, we can use a dangling pointer
            let dangling_ptr = std::ptr::NonNull::<Self>::dangling().as_ptr();
            // SAFETY: For zero-sized types, Box::from_raw with a dangling pointer is safe
            // because zero-sized types don't require actual memory allocation
            return Ok(unsafe { Box::from_raw(dangling_ptr) });
        }

        let type_name = std::any::type_name::<Self>();

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
                return Err(alloc_err!(OutOfMemory {
                    required: layout.size(),
                    alignment: layout.align(),
                })
                .with_type_name(type_name)
                .build());
            }

            let obj_ptr = ptr as *mut Self;
            Ok(Box::from_raw(obj_ptr))
        }
    }
}

impl<T: AllocZeroed> AllocZeroedBoxed for T {}
