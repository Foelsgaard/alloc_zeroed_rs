use super::std;
use std::string::String;

use super::{AllocError, AllocErrorKind};

impl AllocError {
    pub fn suggestion(&self) -> Option<String> {
        use AllocErrorKind::*;

        match self.kind() {
            BufferTooSmall {
                required,
                available,
                ..
            } => Some(std::format!(
                "Increase buffer size by at least {} bytes",
                required - available
            )),

            AlignmentFailed {
                required_alignment, ..
            } => Some(std::format!(
                "Use a buffer aligned to {} bytes",
                required_alignment
            )),

            _ => None,
        }
    }
}
