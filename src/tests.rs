use super::*;

#[test]
fn test_primitive_allocation() {
    let boxed_int = alloc_zeroed::<u32>().unwrap();
    assert_eq!(*boxed_int, 0);

    let boxed_float = alloc_zeroed::<f64>().unwrap();
    assert_eq!(*boxed_float, 0.0);

    let boxed_bool = alloc_zeroed::<bool>().unwrap();
    assert!(!(*boxed_bool));
}

#[test]
fn test_array_allocation() {
    let boxed_array = alloc_zeroed::<[u32; 10]>().unwrap();
    assert_eq!(*boxed_array, [0; 10]);
}

#[test]
fn test_tuple_allocation() {
    let boxed_tuple = alloc_zeroed::<(u32, u8, bool)>().unwrap();
    assert_eq!(*boxed_tuple, (0, 0, false));
}

#[test]
fn test_zst_allocation() {
    #[derive(Debug, PartialEq)]
    struct Zst;

    unsafe impl AllocZeroed for Zst {}

    let boxed_zst = alloc_zeroed::<Zst>().unwrap();
    assert_eq!(*boxed_zst, Zst);
}

#[test]
fn test_custom_struct_allocation() {
    #[derive(Debug, PartialEq)]
    struct Point {
        x: f64,
        y: f64,
        z: f64,
    }

    unsafe impl AllocZeroed for Point {}

    let boxed_point = alloc_zeroed::<Point>().unwrap();
    assert_eq!(
        *boxed_point,
        Point {
            x: 0.0,
            y: 0.0,
            z: 0.0
        }
    );
}

#[test]
fn test_insufficient_memory() {
    // Test with a buffer that's too small
    let mut small_buffer = [0u8; 4]; // Too small for a u64
    let result = u64::alloc_zeroed(&mut small_buffer);
    assert!(matches!(
        result,
        Err(AllocError::BufferTooSmall {
            required: 8,
            available: _,
            alignment: _
        })
    ));
}

#[test]
fn test_exact_size_buffer() {
    // Test with a buffer that's exactly the right size
    let mut exact_buffer = [0u8; std::mem::size_of::<u32>()];
    let result = u32::alloc_zeroed(&mut exact_buffer);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 0);
}

#[test]
fn test_alignment_requirements() {
    // Test with a type that has specific alignment requirements
    #[repr(align(16))]
    struct Aligned(u32);

    unsafe impl AllocZeroed for Aligned {}

    let boxed_aligned = alloc_zeroed::<Aligned>().unwrap();
    assert_eq!(boxed_aligned.0, 0);

    // Check that the pointer is properly aligned
    let ptr = &*boxed_aligned as *const _ as *const u8 as usize;
    assert_eq!(ptr % 16, 0);
}

#[test]
fn test_alloc_error_display() {
    // Test BufferTooSmall
    assert_eq!(
        AllocError::BufferTooSmall {
            required: 100,
            available: 50,
            alignment: 8
        }
        .to_string(),
        "required 100 bytes (with 8 alignment) but only 50 bytes available in buffer"
    );

    // Test OutOfMemory
    assert_eq!(
        AllocError::OutOfMemory {
            required: 1024,
            alignment: 16
        }
        .to_string(),
        "out of memory: required 1024 bytes with 16 alignment"
    );

    // Test AlignmentFailed
    assert_eq!(
        AllocError::AlignmentFailed {
            required_alignment: 16,
            address: 0x1001
        }
        .to_string(),
        "could not align address 4097 to required alignment 16"
    );

    // Test InvalidLayout
    assert_eq!(
        AllocError::InvalidLayout {
            size: 0,
            alignment: 16
        }
        .to_string(),
        "invalid layout: size=0, alignment=16"
    );
}

#[test]
fn test_alloc_error_debug() {
    // Test BufferTooSmall
    assert!(
        format!(
            "{:?}",
            AllocError::BufferTooSmall {
                required: 100,
                available: 50,
                alignment: 8
            }
        )
        .contains("BufferTooSmall")
    );

    // Test OutOfMemory
    assert!(
        format!(
            "{:?}",
            AllocError::OutOfMemory {
                required: 1024,
                alignment: 16
            }
        )
        .contains("OutOfMemory")
    );

    // Test AlignmentFailed
    assert!(
        format!(
            "{:?}",
            AllocError::AlignmentFailed {
                required_alignment: 16,
                address: 0x1001
            }
        )
        .contains("AlignmentFailed")
    );

    // Test InvalidLayout
    assert!(
        format!(
            "{:?}",
            AllocError::InvalidLayout {
                size: 0,
                alignment: 16
            }
        )
        .contains("InvalidLayout")
    );
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_alloc_error_clone_and_partial_eq() {
    // Test that errors can be cloned and compared
    let err1 = AllocError::BufferTooSmall {
        required: 100,
        available: 50,
        alignment: 8,
    };
    let err2 = err1.clone();
    assert_eq!(err1, err2);

    let err3 = AllocError::OutOfMemory {
        required: 1024,
        alignment: 16,
    };
    assert_ne!(err1, err3);

    // Test that different instances with same values are equal
    let err4 = AllocError::BufferTooSmall {
        required: 100,
        available: 50,
        alignment: 8,
    };
    assert_eq!(err1, err4);

    // Test that different instances with different values are not equal
    let err5 = AllocError::BufferTooSmall {
        required: 200, // Different required size
        available: 50,
        alignment: 8,
    };
    assert_ne!(err1, err5);
}
