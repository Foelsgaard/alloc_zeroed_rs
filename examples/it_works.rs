use alloc_zeroed_macros::AllocZeroed;
use alloc_zeroed::{AllocZeroed, alloc_zeroed};

#[derive(AllocZeroed)]
struct LargeData {
    _values: [f64; 1_000_000],
    _metadata: u32,
    _valid: bool,
}

// Now you can use allocate_zeroed with LargeData
fn main() {
    if let Some(_large_data) = alloc_zeroed::<LargeData>() {
        println!("Successfully allocated large data structure");
    }
}
