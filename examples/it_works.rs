use alloc_zeroed::AllocZeroed;

#[derive(AllocZeroed)]
struct LargeData {
    _values: [f64; 1_000_000],
    _metadata: u32,
    _valid: bool,
    _tuple: (u32, u8, u16, f32),
}

// Now you can use allocate_zeroed with LargeData
fn main() {
    match LargeData::alloc_zeroed_boxed() {
        Ok(_large_data) => {
            println!("Successfully allocated large data structure");
            // Use large_data here
        }
        Err(e) => {
            eprintln!("Failed to allocate: {}", e);
        }
    }
}
