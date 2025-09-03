# AllocZeroed

A Rust library for safe zero-initialized allocation of types, especially useful for large objects that can't fit on the stack.

## Features

- **Zero-initialized allocation**: Safely allocate and initialize types with all-zero bit patterns
- **Stack overflow prevention**: Perfect for large types that would normally cause stack overflows
- **Derive macro**: Automatically implement `AllocZeroed` for your structs (optional feature)
- **Comprehensive type support**: Built-in implementations for primitives, arrays, and tuples

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
alloc_zeroed = "0.1"
```

For derive macro support, enable the `derive` feature:

```toml
[dependencies]
alloc_zeroed = { version = "0.1", features = ["derive"] }
```

## Usage

### Basic Usage

```rust
use alloc_zeroed::{AllocZeroed, alloc_zeroed};

// For types that implement AllocZeroed
if let Some(my_data) = alloc_zeroed::<MyType>() {
    // Use your zero-initialized data
}
```

### Using the Derive Macro

```rust
use alloc_zeroed::AllocZeroed;

#[derive(AllocZeroed)]
struct LargeData {
    values: [f64; 1_000_000],
    metadata: u32,
    valid: bool,
    tuple_field: (u32, u8, u16, f32),
}

fn main() {
    if let Some(large_data) = alloc_zeroed::<LargeData>() {
        println!("Successfully allocated large data structure");
        // large_data is now zero-initialized and ready to use
    }
}
```

### Manual Implementation

For types that can't use the derive macro, you can manually implement `AllocZeroed`:

```rust
use alloc_zeroed::AllocZeroed;

struct CustomType {
    // your fields
}

// SAFETY: Ensure all-zero bit pattern is valid for your type
unsafe impl AllocZeroed for CustomType {
    // Optional: override the default implementation if needed
}
```

## Safety

The `AllocZeroed` trait is marked as `unsafe` because not all types can be safely initialized with zeros. Before implementing this trait for your type, ensure that:

1. All-bit-zero is a valid representation for your type
2. The type doesn't have any invariants that would be violated by zero initialization
3. All fields of the type also satisfy these conditions (checked automatically by the derive macro)

## Examples

See the `examples` directory for complete working examples.

## Development

### Code Coverage

To generate a code coverage report locally:

```bash
# Install the coverage tool (if not already installed)
cargo install cargo-llvm-cov

# Generate and open an HTML coverage report
cargo llvm-cov --all-features --workspace --html --open
```

## License

Licensed under either of:
- Apache License, Version 2.0
- MIT license

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.
