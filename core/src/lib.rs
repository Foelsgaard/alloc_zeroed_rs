//! A safe, zero-initialized allocation library for Rust with support for both `no_std` and `std` environments.
//!
//! This crate provides traits and utilities for safely allocating and initializing types with
//! all-zero bit patterns, which is particularly useful for:
//!
//! - Large types that cannot fit on the stack
//! - Embedded systems and `no_std` environments
//! - Scenarios where zero-initialization is required for safety
//!
//! # Features
//!
//! - **`no_std` compatible**: Core functionality works without the standard library
//! - **Zero-sized type support**: Proper handling of types with no allocated storage
//! - **Alignment-aware**: Correctly handles types with specific alignment requirements
//! - **Detailed error reporting**: Rich error information for allocation failures
//! - **Derive macro**: Automatic implementation for structs with `#[derive(AllocZeroed)]`
//! - **Standard library integration**: Optional `std` feature for `Box`-based allocation
//!
//! # Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! alloc_zeroed = "0.1"
//! ```
//!
//! For `Box`-based allocation, enable the `std` feature:
//!
//! ```toml
//! [dependencies]
//! alloc_zeroed = { version = "0.1", features = ["std"] }
//! ```
//!
//! For derive macro support, enable the `derive` feature:
//!
//! ```toml
//! [dependencies]
//! alloc_zeroed = { version = "0.1", features = ["derive"] }
//! ```
//!
//! # Examples
//!
//! ## Buffer-based allocation (`no_std` compatible)
//!
//! ```rust
//! use alloc_zeroed::AllocZeroed;
//!
//! #[derive(AllocZeroed)]
//! struct SensorData {
//!     values: [f32; 1000],
//!     timestamp: u64,
//!     valid: bool,
//! }
//!
//! let mut buffer = [0u8; 4096];
//! let sensor_data = SensorData::alloc_zeroed(&mut buffer).unwrap();
//! ```
//!
//! ## Box-based allocation (requires `std` feature)
//!
//! ```rust
//! use alloc_zeroed::{AllocZeroed, AllocZeroedBoxed};
//!
//! #[derive(AllocZeroed)]
//! struct LargeData {
//!     matrix: [[f64; 100]; 100],
//!     metadata: u32,
//! }
//!
//! let large_data = LargeData::alloc_zeroed_boxed().unwrap();
//! ```
//!
//! # Safety
//!
//! The `AllocZeroed` trait is marked as `unsafe` because not all types can be safely
//! initialized with zeros. Before implementing this trait for your type, ensure that:
//!
//! 1. All-bit-zero is a valid representation for your type
//! 2. The type doesn't have any invariants that would be violated by zero initialization
//! 3. All fields of the type also satisfy these conditions
//!
//! The derive macro automatically checks that all field types implement `AllocZeroed`,
//! providing a compile-time guarantee of safety for derived implementations.
//!
//! # Crate Organization
//!
//! - Core functionality (`AllocZeroed` trait) is available in `no_std` environments
//! - Standard library integration (`AllocZeroedBoxed` trait) is gated behind the `std` feature
//! - Derive macro support is gated behind the `derive` feature

#![no_std]

mod core;

pub use crate::core::{AllocError, AllocErrorKind, AllocZeroed};

#[cfg(feature = "std")]
mod std;

#[cfg(feature = "std")]
pub use crate::std::AllocZeroedBoxed;

#[cfg(test)]
mod tests;
