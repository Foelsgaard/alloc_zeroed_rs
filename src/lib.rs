#[cfg(feature = "derive")]
pub use alloc_zeroed_macros::AllocZeroed;

#[macro_use]
mod error;
mod implementations;

pub use error::{AllocError, AllocErrorKind};

/// # Safety
/// All-zero pattern must be a valid value of type.
pub unsafe trait AllocZeroed: Sized {
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
            let dangling_ptr = std::ptr::NonNull::<Self>::dangling().as_ptr();
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

    /// # Examples
    ///
    /// ```
    /// use alloc_zeroed::AllocZeroed;
    ///
    /// let value = u32::alloc_zeroed_boxed().unwrap();
    /// assert_eq!(*value, 0);
    /// ```
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

#[cfg(test)]
mod tests;
