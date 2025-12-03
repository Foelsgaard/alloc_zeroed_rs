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
        let (element, _) = Self::alloc_zeroed_with_remainder(mem)?;

        Ok(element)
    }

    /// Allocates and zero-initializes an instance of `Self` in the provided buffer, returning the remainder.
    ///
    /// This method allocates memory for a single instance of `Self` within the given byte buffer,
    /// ensuring proper alignment and zero-initialization. Unlike [`alloc_zeroed`], this method
    /// returns both the allocated object and the remaining unused portion of the buffer, allowing
    /// for efficient sequential allocations.
    ///
    /// # Parameters
    ///
    /// * `mem` - A mutable byte slice where the object will be allocated. The buffer must
    ///           be large enough to accommodate the type's size and alignment requirements.
    ///
    /// # Returns
    ///
    /// * `Ok((&mut Self, &mut [u8]))` - A tuple containing:
    ///   - A mutable reference to the zero-initialized object
    ///   - The remaining bytes in the buffer after allocation
    /// * `Err(AllocError)` - An error describing why allocation failed
    ///
    /// # Errors
    ///
    /// Returns `AllocError` in the following cases:
    /// * `AllocError::BufferTooSmall` - The buffer doesn't have enough space for the type
    /// * `AllocError::AlignmentFailed` - The buffer cannot be aligned to the type's requirements
    ///
    /// # Safety
    ///
    /// This method is unsafe because it assumes that an all-zero bit pattern is a valid
    /// representation for the type `Self`. Implementors must ensure this invariant holds.
    ///
    /// # Examples
    ///
    /// ## Single allocation with remainder
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
    /// let (point, remainder) = Point::alloc_zeroed_with_remainder(&mut buffer).unwrap();
    /// assert_eq!(point.x, 0.0);
    /// assert_eq!(point.y, 0.0);
    /// assert!(!remainder.is_empty());
    /// ```
    ///
    /// ## Chained allocations
    /// ```
    /// use alloc_zeroed::AllocZeroed;
    ///
    /// let mut buffer = [0u8; 1024];
    ///
    /// // Allocate multiple u32 values sequentially
    /// let (first, remainder1) = u32::alloc_zeroed_with_remainder(&mut buffer).unwrap();
    /// let (second, remainder2) = u32::alloc_zeroed_with_remainder(remainder1).unwrap();
    /// let (third, final_remainder) = u32::alloc_zeroed_with_remainder(remainder2).unwrap();
    ///
    /// *first = 1;
    /// *second = 2;
    /// *third = 3;
    /// ```
    ///
    /// # Zero-Sized Types
    ///
    /// For zero-sized types (ZSTs), this method always succeeds and returns the original buffer
    /// as the remainder, as ZSTs don't require actual memory allocation.
    ///
    /// # See Also
    ///
    /// * [`alloc_zeroed`] - For simple allocation without remainder
    /// * [`alloc_zeroed_slice_with_remainder`] - For allocating multiple elements with remainder
    ///
    /// [`alloc_zeroed`]: AllocZeroed::alloc_zeroed
    /// [`alloc_zeroed_slice_with_remainder`]: AllocZeroed::alloc_zeroed_slice_with_remainder
    fn alloc_zeroed_with_remainder(mem: &mut [u8]) -> Result<(&mut Self, &mut [u8]), AllocError> {
        let (slice, remainder) = Self::alloc_zeroed_slice_with_remainder(mem, 1)?;

        Ok((slice.first_mut().unwrap(), remainder))
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
        let size = size_of::<Self>();
        let align = align_of::<Self>();
        let mem_ptr = mem.as_mut_ptr();
        let offset = mem_ptr.align_offset(align);
        let available_bytes = mem.len().saturating_sub(offset);

        // Calculate how many complete items we can fit
        let count = if size == 0 {
            usize::MAX
        } else {
            available_bytes / size
        };

        let (slice, _) = Self::alloc_zeroed_slice_with_remainder(mem, count)?;

        Ok(slice)
    }

    /// Allocates a slice of zero-initialized `Self` values from the buffer, returning the remainder.
    ///
    /// This method allocates memory for multiple instances of `Self` within the given byte buffer,
    /// ensuring proper alignment and zero-initialization. It returns both the allocated slice and
    /// the remaining unused portion of the buffer, allowing for efficient memory management when
    /// allocating arrays or collections.
    ///
    /// # Parameters
    ///
    /// * `mem` - A mutable byte slice where the objects will be allocated
    /// * `count` - The number of elements to allocate in the slice
    ///
    /// # Returns
    ///
    /// * `Ok((&mut [Self], &mut [u8]))` - A tuple containing:
    ///   - A mutable slice of zero-initialized objects
    ///   - The remaining bytes in the buffer after allocation
    /// * `Err(AllocError)` - An error describing why allocation failed
    ///
    /// # Errors
    ///
    /// Returns `AllocError` in the following cases:
    /// * `AllocError::BufferTooSmall` - The buffer doesn't have enough space for all requested elements
    /// * `AllocError::AlignmentFailed` - The buffer cannot be aligned to the type's requirements
    ///
    /// # Safety
    ///
    /// This method is unsafe because it assumes that an all-zero bit pattern is a valid
    /// representation for the type `Self`. Implementors must ensure this invariant holds.
    ///
    /// # Examples
    ///
    /// ## Allocating a fixed number of elements
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
    /// let (points, remainder) = Point::alloc_zeroed_slice_with_remainder(&mut buffer, 5).unwrap();
    /// assert_eq!(points.len(), 5);
    /// assert_eq!(points[0].x, 0.0);
    /// assert!(!remainder.is_empty());
    /// ```
    ///
    /// ## Mixed allocation types
    /// ```
    /// use alloc_zeroed::AllocZeroed;
    ///
    /// let mut buffer = [0u8; 1024];
    ///
    /// // First allocate some u32 values
    /// let (numbers, remainder) = u32::alloc_zeroed_slice_with_remainder(&mut buffer, 10).unwrap();
    /// assert_eq!(numbers.len(), 10);
    ///
    /// // Then allocate some u64 values from the remainder
    /// let (large_numbers, final_remainder) = u64::alloc_zeroed_slice_with_remainder(remainder, 5).unwrap();
    /// assert_eq!(large_numbers.len(), 5);
    /// ```
    ///
    /// ## Calculating maximum possible allocation
    /// ```
    /// use alloc_zeroed::AllocZeroed;
    /// use core::mem::size_of;
    ///
    /// let mut buffer = [0u8; 1024];
    /// let element_size = size_of::<u32>();
    /// let max_count = buffer.len() / element_size; // Rough estimate
    ///
    /// // Try to allocate as many as possible (may fail due to alignment)
    /// match u32::alloc_zeroed_slice_with_remainder(&mut buffer, max_count) {
    ///     Ok((slice, remainder)) => {
    ///         println!("Allocated {} u32 values, {} bytes remaining", slice.len(), remainder.len());
    ///     }
    ///     Err(_) => {
    ///         // Try with fewer elements due to alignment constraints
    ///         let (slice, remainder) = u32::alloc_zeroed_slice_with_remainder(&mut buffer, max_count - 1).unwrap();
    ///     }
    /// }
    /// ```
    ///
    /// # Zero-Sized Types
    ///
    /// For zero-sized types (ZSTs), this method always succeeds and returns a slice of length
    /// `usize::MAX` along with the original buffer as remainder, as ZSTs don't require actual
    /// memory allocation.
    ///
    /// # Performance Notes
    ///
    /// The entire allocated slice is zero-initialized in a single operation, which is typically
    /// more efficient than allocating elements individually.
    ///
    /// # See Also
    ///
    /// * [`alloc_zeroed_slice`] - For allocating the maximum possible slice without remainder
    /// * [`alloc_zeroed_with_remainder`] - For allocating single elements with remainder
    ///
    /// [`alloc_zeroed_slice`]: AllocZeroed::alloc_zeroed_slice
    /// [`alloc_zeroed_with_remainder`]: AllocZeroed::alloc_zeroed_with_remainder
    fn alloc_zeroed_slice_with_remainder(
        mem: &mut [u8],
        count: usize,
    ) -> Result<(&mut [Self], &mut [u8]), AllocError> {
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
            return Ok((slice, mem));
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
        let total_bytes = size * count;
        if available_bytes < total_bytes {
            return Err(AllocError::builder(AllocErrorKind::BufferTooSmall {
                required: total_bytes,
                available: available_bytes,
                alignment: align,
            })
            .build());
        }

        let (_before, after) = mem.split_at_mut(offset);
        let (alloc_slice, remainder) = after.split_at_mut(total_bytes);

        // Zero the memory
        alloc_slice.fill(0);

        // SAFETY: We've ensured the pointer is properly aligned and there's enough space
        // The memory has been zeroed, which is valid for T (guaranteed by AllocZeroed trait bound)
        unsafe {
            let ptr = alloc_slice.as_mut_ptr() as *mut Self;
            Ok((core::slice::from_raw_parts_mut(ptr, count), remainder))
        }
    }
}
