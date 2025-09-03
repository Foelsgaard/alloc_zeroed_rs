# AllocZeroed

A safe, zero-initialized allocation library for Rust with support for both `no_std` and `std` environments.

This crate provides traits and utilities for safely allocating and initializing types with all-zero bit patterns, which is particularly useful for:

- Large types that cannot fit on the stack
- Embedded systems and `no_std` environments  
- Scenarios where zero-initialization is required for safety

## Features

- **`no_std` compatible**: Core functionality works without the standard library
- **Zero-sized type support**: Proper handling of types with no allocated storage
- **Alignment-aware**: Correctly handles types with specific alignment requirements
- **Detailed error reporting**: Rich error information for allocation failures
- **Derive macro**: Automatic implementation for structs with `#[derive(AllocZeroed)]`
- **Standard library integration**: Optional `std` feature for `Box`-based allocation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
alloc_zeroed = "0.1"
```

For `Box`-based allocation, enable the `std` feature:

```toml
[dependencies]
alloc_zeroed = { version = "0.1", features = ["std"] }
```

For derive macro support, enable the `derive` feature:

```toml
[dependencies]
alloc_zeroed = { version = "0.1", features = ["derive"] }
```

## Usage

### Buffer-based Allocation (`no_std` compatible)

```rust
use alloc_zeroed::AllocZeroed;

#[derive(AllocZeroed)]
struct SensorData {
    values: [f32; 1000],
    timestamp: u64,
    valid: bool,
}

fn main() {
    let mut buffer = [0u8; 4096];
    let sensor_data = SensorData::alloc_zeroed(&mut buffer).unwrap();
    // Use sensor_data...
}
```

### Box-based Allocation (requires `std` feature)

```rust
use alloc_zeroed::{AllocZeroed, AllocZeroedBoxed};

#[derive(AllocZeroed)]
struct LargeData {
    matrix: [[f64; 100]; 100],
    metadata: u32,
}

fn main() {
    let large_data = LargeData::alloc_zeroed_boxed().unwrap();
    // Use large_data...
}
```

## API Overview

### Core Trait (`no_std` compatible)

```rust
pub unsafe trait AllocZeroed: Sized {
    fn alloc_zeroed(mem: &mut [u8]) -> Result<&mut Self, AllocError>;
}
```

### Std Trait (requires `std` feature)

```rust
pub trait AllocZeroedBoxed: AllocZeroed {
    fn alloc_zeroed_boxed() -> Result<Box<Self>, AllocError>;
}
```

### Error Handling

The crate provides detailed error information through the `AllocError` type:

```rust
#[derive(Debug, Clone, Copy)]
pub struct AllocError {
    // ... error details with context information
}

impl core::fmt::Display for AllocError { ... }
```

## Safety

The `AllocZeroed` trait is marked as `unsafe` because not all types can be safely initialized with zeros. Before implementing this trait for your type, ensure that:

1. All-bit-zero is a valid representation for your type
2. The type doesn't have any invariants that would be violated by zero initialization
3. All fields of the type also satisfy these conditions

The derive macro automatically checks that all field types implement `AllocZeroed`, providing a compile-time guarantee of safety for derived implementations.

## Examples

See the `examples` directory for complete working examples:

- `alloc_zeroed.rs`: Buffer-based allocation example
- `alloc_zeroed_boxed.rs`: Box-based allocation example

## License

Licensed under either of:
- Apache License, Version 2.0
- MIT license

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.
