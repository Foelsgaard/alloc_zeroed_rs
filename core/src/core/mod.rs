#[cfg(feature = "derive")]
pub use alloc_zeroed_macros::AllocZeroed;

#[macro_use]
pub mod error;
pub mod implementations;

pub use error::{AllocError, AllocErrorKind};

/// # Safety
/// All-zero pattern must be a valid value of type.
pub unsafe trait AllocZeroed: Sized {
    /// Allocates and zero-initializes an instance of `Self` in the provided buffer.
    ///
    /// This method attempts to allocate memory for `Self` within the given byte buffer,
    /// ensuring proper alignment and zero-initializing the allocated memory.
    ///
    /// # Parameters
    ///
    /// * `mem` - A mutable byte slice where the object will be allocated. The buffer must
    ///           be large enough to accommodate the type's size and alignment requirements.
    ///
    /// # Returns
    ///
    /// * `Ok(&mut Self)` - A mutable reference to the zero-initialized object if allocation succeeds.
    /// * `Err(AllocError)` - An error describing why allocation failed (insufficient space,
    ///                       alignment issues, or invalid layout).
    ///
    /// # Errors
    ///
    /// Returns `AllocError` in the following cases:
    /// * `AllocError::BufferTooSmall` - The buffer doesn't have enough space for the type
    /// * `AllocError::AlignmentFailed` - The buffer cannot be aligned to the type's requirements
    /// * `AllocError::InvalidLayout` - The type has an invalid size or alignment combination
    ///
    /// # Safety
    ///
    /// This method is unsafe because it assumes that an all-zero bit pattern is a valid
    /// representation for the type `Self`. Implementors must ensure this invariant holds.
    ///
    /// # Examples
    ///
    /// ```
    /// use alloc_zeroed::AllocZeroed;
    ///
    /// #[derive(AllocZeroed)]
    /// struct Point {
    ///     x: f64,
    ///     y: f64,
    /// }
    ///
    /// let mut buffer = [0u8; 1024];
    /// let point = Point::alloc_zeroed(&mut buffer).unwrap();
    /// assert_eq!(point.x, 0.0);
    /// assert_eq!(point.y, 0.0);
    /// ```
    ///
    /// # Zero-Sized Types
    ///
    /// For zero-sized types (ZSTs), this method always succeeds and returns a dangling pointer,
    /// as ZSTs don't require actual memory allocation.
    fn alloc_zeroed(mem: &mut [u8]) -> Result<&mut Self, AllocError> {
        use AllocErrorKind::*;
        use core::mem;

        let size = mem::size_of::<Self>();
        let align = mem::align_of::<Self>();
        let len = mem.len();

        let mem_ptr = mem.as_mut_ptr();
        let offset = mem_ptr.align_offset(align);

        // Handle zero-sized types
        if size == 0 {
            // SAFETY: Zero-sized types don't require actual memory
            let dangling_ptr = core::ptr::NonNull::<Self>::dangling().as_ptr();
            return unsafe { Ok(&mut *dangling_ptr) };
        }

        let type_name = core::any::type_name::<Self>();

        if offset == usize::MAX {
            return Err(alloc_err!(AlignmentFailed {
                required_alignment: align,
                address: mem_ptr as usize,
            })
            .with_type_name(type_name)
            .build());
        }

        if size > len.saturating_sub(offset) {
            return Err(alloc_err!(BufferTooSmall {
                required: size,
                available: len.saturating_sub(offset),
                alignment: align,
            })
            .with_type_name(type_name)
            .build());
        }

        // SAFETY: We've checked that the offset is valid and there's enough space
        let ptr = unsafe { mem_ptr.add(offset) as *mut Self };

        // SAFETY: We've ensured the pointer is properly aligned and there's enough space
        unsafe {
            ptr.write_bytes(0, 1);
            Ok(&mut *ptr)
        }
    }

    /// Allocates the largest possible slice of zero-initialized `T` values from a byte buffer
    ///
    /// This method attempts to allocate a slice of `T` values from the provided byte buffer,
    /// ensuring proper alignment and zero-initialization. It returns the largest possible
    /// contiguous slice that fits in the available space after alignment requirements are met.
    ///
    /// # Safety
    /// The same safety requirements as [`alloc_zeroed`] apply - the all-zero bit pattern must
    /// be valid for type `T`. This is guaranteed by the [`AllocZeroed`] trait bound.
    ///
    /// # Behavior for Zero-Sized Types (ZSTs)
    /// For zero-sized types, this returns a slice of length [`usize::MAX`] since ZSTs require
    /// no storage and can be created in unlimited quantities from any aligned pointer.
    ///
    /// # Errors
    /// Returns [`AllocError`] if:
    /// - The buffer cannot be aligned to `T`'s alignment requirements
    /// - The available space after alignment is smaller than the size of one `T`
    ///
    /// # Examples
    /// ```
    /// # use alloc_zeroed::AllocZeroed;
    /// # use core::mem::size_of;
    /// let mut buffer = [0u8; 1024];
    /// let slice = u32::alloc_zeroed_slice(&mut buffer).unwrap();
    /// assert!(slice.len() >= 256); // At least 256 u32s in 1KB (considering alignment)
    /// ```
    ///
    /// [`alloc_zeroed`]: AllocZeroed::alloc_zeroed
    fn alloc_zeroed_slice(mem: &mut [u8]) -> Result<&mut [Self], AllocError> {
        use core::mem::{align_of, size_of};

        let size = size_of::<Self>();
        let align = align_of::<Self>();

        // Handle zero-sized types
        if size == 0 {
            // For ZSTs, we can create as many as will fit in usize::MAX
            let slice = unsafe {
                core::slice::from_raw_parts_mut(
                    core::ptr::NonNull::<Self>::dangling().as_ptr(),
                    usize::MAX,
                )
            };
            return Ok(slice);
        }

        let mem_ptr = mem.as_mut_ptr();
        let offset = mem_ptr.align_offset(align);

        if offset == usize::MAX {
            return Err(AllocError::builder(AllocErrorKind::AlignmentFailed {
                required_alignment: align,
                address: mem_ptr as usize,
            })
            .build());
        }

        let available_bytes = mem.len().saturating_sub(offset);
        if available_bytes < size {
            return Err(AllocError::builder(AllocErrorKind::BufferTooSmall {
                required: size,
                available: available_bytes,
                alignment: align,
            })
            .build());
        }

        // Calculate how many complete items we can fit
        let count = available_bytes / size;
        let total_bytes = count * size;

        // Get the slice for the allocation
        let alloc_slice = &mut mem[offset..offset + total_bytes];

        // Zero the memory
        alloc_slice.fill(0);

        // SAFETY: We've ensured the pointer is properly aligned and there's enough space
        // The memory has been zeroed, which is valid for T (guaranteed by AllocZeroed trait bound)
        unsafe {
            let ptr = alloc_slice.as_mut_ptr() as *mut Self;
            Ok(core::slice::from_raw_parts_mut(ptr, count))
        }
    }
}
