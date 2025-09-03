#[derive(Debug, Clone, Copy)]
pub struct AllocError {
    kind: AllocErrorKind,
    type_name: Option<&'static str>,
    file: Option<&'static str>,
    line: Option<u32>,
    additional_context: Option<&'static str>,
}

impl AllocError {
    pub fn builder(kind: AllocErrorKind) -> AllocErrorBuilder {
        AllocErrorBuilder::new(kind)
    }

    pub fn kind(&self) -> AllocErrorKind {
        self.kind
    }

    pub fn type_name(&self) -> Option<&'static str> {
        self.type_name
    }

    pub fn location(&self) -> Option<(&'static str, u32)> {
        self.file.zip(self.line)
    }

    pub fn additional_context(&self) -> Option<&'static str> {
        self.additional_context
    }

    // Convenience methods for common error types
    pub fn buffer_too_small(
        required: usize,
        available: usize,
        alignment: usize,
    ) -> AllocErrorBuilder {
        AllocErrorBuilder::new(AllocErrorKind::BufferTooSmall {
            required,
            available,
            alignment,
        })
    }

    pub fn out_of_memory(required: usize, alignment: usize) -> AllocErrorBuilder {
        AllocErrorBuilder::new(AllocErrorKind::OutOfMemory {
            required,
            alignment,
        })
    }

    pub fn is_insufficient_memory(&self) -> bool {
        use AllocErrorKind::*;

        matches!(self.kind, BufferTooSmall { .. } | OutOfMemory { .. })
    }

    pub fn required_size(&self) -> Option<usize> {
        use AllocErrorKind::*;

        match self.kind {
            BufferTooSmall { required, .. } => Some(required),
            OutOfMemory { required, .. } => Some(required),
            _ => None,
        }
    }
}

impl AllocError {
    pub fn suggestion(&self) -> Option<String> {
        use AllocErrorKind::*;

        match self.kind {
            BufferTooSmall {
                required,
                available,
                ..
            } => Some(format!(
                "Increase buffer size by at least {} bytes",
                required - available
            )),

            AlignmentFailed {
                required_alignment, ..
            } => Some(format!(
                "Use a buffer aligned to {} bytes",
                required_alignment
            )),

            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocErrorKind {
    BufferTooSmall {
        required: usize,
        available: usize,
        alignment: usize,
    },
    OutOfMemory {
        required: usize,
        alignment: usize,
    },
    AlignmentFailed {
        required_alignment: usize,
        address: usize,
    },
    InvalidLayout {
        size: usize,
        alignment: usize,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct AllocErrorBuilder {
    kind: AllocErrorKind,
    type_name: Option<&'static str>,
    file: Option<&'static str>,
    line: Option<u32>,
    additional_context: Option<&'static str>,
}

impl AllocErrorBuilder {
    pub fn new(kind: AllocErrorKind) -> Self {
        Self {
            kind,
            type_name: None,
            file: None,
            line: None,
            additional_context: None,
        }
    }

    pub fn with_type_name(mut self, type_name: &'static str) -> Self {
        self.type_name = Some(type_name);
        self
    }

    pub fn with_location(mut self, file: &'static str, line: u32) -> Self {
        self.file = Some(file);
        self.line = Some(line);
        self
    }

    pub fn with_context(mut self, context: &'static str) -> Self {
        self.additional_context = Some(context);
        self
    }

    pub fn build(self) -> AllocError {
        AllocError {
            kind: self.kind,
            type_name: self.type_name,
            file: self.file,
            line: self.line,
            additional_context: self.additional_context,
        }
    }
}

impl std::fmt::Display for AllocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Write the base error message
        match self.kind {
            AllocErrorKind::BufferTooSmall {
                required,
                available,
                alignment,
            } => write!(
                f,
                "required {} bytes (with {} alignment) but only {} bytes available",
                required, alignment, available
            ),
            AllocErrorKind::OutOfMemory {
                required,
                alignment,
            } => write!(
                f,
                "out of memory: required {} bytes with {} alignment",
                required, alignment
            ),
            AllocErrorKind::AlignmentFailed {
                required_alignment,
                address,
            } => write!(
                f,
                "could not align address {} to required alignment {}",
                address, required_alignment
            ),
            AllocErrorKind::InvalidLayout { size, alignment } => {
                write!(f, "invalid layout: size={}, alignment={}", size, alignment)
            }
        }?;

        // Add context information if available
        if let Some(type_name) = self.type_name {
            write!(f, " (type: {})", type_name)?;
        }

        if let Some((file, line)) = self.location() {
            write!(f, " (at {}:{})", file, line)?;
        }

        if let Some(context) = self.additional_context {
            write!(f, " (context: {})", context)?;
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! alloc_err {
    ($kind:expr) => {
        AllocError::builder($kind).with_location(file!(), line!())
    };
}
