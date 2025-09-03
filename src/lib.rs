#![no_std]

mod core;

pub use crate::core::{AllocError, AllocErrorKind, AllocZeroed};

#[cfg(feature = "std")]
mod std;

#[cfg(feature = "std")]
pub use crate::std::AllocZeroedBoxed;

#[cfg(test)]
mod tests;
