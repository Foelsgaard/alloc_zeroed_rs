#[cfg(feature = "derive")]
pub use alloc_zeroed_macros::AllocZeroed;

#[macro_use]
pub mod error;
pub mod implementations;

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
}
