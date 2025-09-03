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
