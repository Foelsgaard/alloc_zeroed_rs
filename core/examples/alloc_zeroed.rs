use alloc_zeroed::AllocZeroed;

#[derive(AllocZeroed)]
struct Data {
    _values: [f64; 1_000],
    _metadata: u32,
    _valid: bool,
    _tuple: (u32, u8, u16, f32),
}

// Now you can use allocate_zeroed with Data
fn main() {
    let mut buf = [0_u8; 0x10000];
    match Data::alloc_zeroed(&mut buf) {
        Ok(_data) => {
            println!("Successfully allocated data structure");
            // Use data here
        }
        Err(e) => {
            eprintln!("Failed to allocate: {}", e);
        }
    }
}
