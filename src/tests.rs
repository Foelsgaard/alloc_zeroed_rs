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
fn test_insufficient_memory() {
    // Test with a buffer that's too small
    let mut small_buffer = [0u8; 4]; // Too small for a u64
    let result = u64::alloc_zeroed(&mut small_buffer);

    // Check that we get the right error kind
    assert!(matches!(
        result.as_ref().map_err(|e| e.kind()),
        Err(AllocErrorKind::BufferTooSmall {
            required: 8,
            available: _,
            alignment: _
        })
    ));

    // Check that the error message contains expected information
    if let Err(e) = result {
        let msg = e.to_string();
        assert!(msg.contains("required 8 bytes"));
        assert!(msg.contains("but only"));
        assert!(msg.contains("bytes available"));
    } else {
        panic!("Expected an error");
    }
}

#[test]
fn test_alloc_error_display() {
    // Test BufferTooSmall without context
    let error = AllocError::builder(AllocErrorKind::BufferTooSmall {
        required: 100,
        available: 50,
        alignment: 8,
    })
    .build();

    let msg = error.to_string();
    assert!(msg.contains("required 100 bytes"));
    assert!(msg.contains("but only 50 bytes available"));
    assert!(msg.contains("8 alignment"));

    // Test OutOfMemory
    let error = AllocError::builder(AllocErrorKind::OutOfMemory {
        required: 1024,
        alignment: 16,
    })
    .build();

    let msg = error.to_string();
    assert!(msg.contains("out of memory"));
    assert!(msg.contains("1024 bytes"));
    assert!(msg.contains("16 alignment"));

    // Test AlignmentFailed
    let error = AllocError::builder(AllocErrorKind::AlignmentFailed {
        required_alignment: 16,
        address: 0x1001,
    })
    .build();

    let msg = error.to_string();
    assert!(msg.contains("could not align address 4097"));
    assert!(msg.contains("required alignment 16"));

    // Test InvalidLayout
    let error = AllocError::builder(AllocErrorKind::InvalidLayout {
        size: 0,
        alignment: 16,
    })
    .build();

    let msg = error.to_string();
    assert!(msg.contains("invalid layout"));
    assert!(msg.contains("size=0"));
    assert!(msg.contains("alignment=16"));
}

#[test]
fn test_alloc_error_debug() {
    // Test that debug output contains the variant name
    let error = AllocError::builder(AllocErrorKind::BufferTooSmall {
        required: 100,
        available: 50,
        alignment: 8,
    })
    .build();

    let debug_output = format!("{:?}", error);
    assert!(debug_output.contains("BufferTooSmall"));
}

#[test]
fn test_alloc_error_builder() {
    // Test that builder sets all fields correctly
    let error = AllocError::builder(AllocErrorKind::BufferTooSmall {
        required: 100,
        available: 50,
        alignment: 8,
    })
    .with_type_name("TestType")
    .with_location("test.rs", 42)
    .with_context("test context")
    .build();

    assert_eq!(error.type_name(), Some("TestType"));
    assert_eq!(error.location(), Some(("test.rs", 42)));
    assert_eq!(error.additional_context(), Some("test context"));
    assert!(matches!(
        error.kind(),
        AllocErrorKind::BufferTooSmall {
            required: 100,
            available: 50,
            alignment: 8
        }
    ));

    // Test that the context appears in the display message
    let msg = error.to_string();
    assert!(msg.contains("TestType"));
    assert!(msg.contains("test.rs:42"));
    assert!(msg.contains("test context"));
}

#[test]
fn test_alloc_error_convenience_methods() {
    // Test convenience methods
    let error = AllocError::buffer_too_small(100, 50, 8)
        .with_type_name("TestType")
        .build();

    assert!(matches!(
        error.kind(),
        AllocErrorKind::BufferTooSmall {
            required: 100,
            available: 50,
            alignment: 8
        }
    ));
    assert_eq!(error.type_name(), Some("TestType"));
}

#[test]
fn test_alloc_error_suggestions() {
    // Test error suggestions
    let error = AllocError::builder(AllocErrorKind::BufferTooSmall {
        required: 100,
        available: 50,
        alignment: 8,
    })
    .build();

    let suggestion = error.suggestion().unwrap();
    assert!(suggestion.contains("Increase buffer size"));
    assert!(suggestion.contains("50 bytes"));

    let error = AllocError::builder(AllocErrorKind::AlignmentFailed {
        required_alignment: 16,
        address: 0x1001,
    })
    .build();

    let suggestion = error.suggestion().unwrap();
    assert!(suggestion.contains("aligned to 16 bytes"));
}

#[test]
fn test_alloc_error_inspection() {
    // Test inspection methods
    let error = AllocError::builder(AllocErrorKind::BufferTooSmall {
        required: 100,
        available: 50,
        alignment: 8,
    })
    .build();

    assert!(error.is_insufficient_memory());
    assert_eq!(error.required_size(), Some(100));

    let error = AllocError::builder(AllocErrorKind::OutOfMemory {
        required: 1024,
        alignment: 16,
    })
    .build();

    assert!(error.is_insufficient_memory());
    assert_eq!(error.required_size(), Some(1024));

    let error = AllocError::builder(AllocErrorKind::AlignmentFailed {
        required_alignment: 16,
        address: 0x1001,
    })
    .build();

    assert!(!error.is_insufficient_memory());
    assert_eq!(error.required_size(), None);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_alloc_error_clone() {
    // Test that errors can be cloned
    let error = AllocError::builder(AllocErrorKind::BufferTooSmall {
        required: 100,
        available: 50,
        alignment: 8,
    })
    .build();

    let cloned = error.clone();

    // Check that they have the same kind
    assert!(matches!(
        error.kind(),
        AllocErrorKind::BufferTooSmall {
            required: 100,
            available: 50,
            alignment: 8
        }
    ));
    assert!(matches!(
        cloned.kind(),
        AllocErrorKind::BufferTooSmall {
            required: 100,
            available: 50,
            alignment: 8
        }
    ));

    // Check that display messages are the same
    assert_eq!(error.to_string(), cloned.to_string());
}

#[test]
fn test_alloc_error_macro() {
    // Test the convenience macro
    let error = alloc_err!(AllocErrorKind::BufferTooSmall {
        required: 100,
        available: 50,
        alignment: 8
    })
    .with_type_name("TestType")
    .build();

    assert!(matches!(
        error.kind(),
        AllocErrorKind::BufferTooSmall {
            required: 100,
            available: 50,
            alignment: 8
        }
    ));
    assert!(error.location().is_some()); // Macro should add location
}
