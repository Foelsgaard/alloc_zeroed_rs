use alloc_zeroed::{AllocZeroed, AllocZeroedBoxed};

#[test]
fn miri_test_primitive() {
    let value = u32::alloc_zeroed_boxed().unwrap();
    assert_eq!(*value, 0);
}

#[test]
fn miri_test_array() {
    let array = <[u32; 100]>::alloc_zeroed_boxed().unwrap();
    for &item in array.iter() {
        assert_eq!(item, 0);
    }
}

#[test]
fn miri_test_custom_struct() {
    #[repr(C)]
    struct TestStruct {
        a: u32,
        b: u64,
        c: [u8; 10],
    }

    unsafe impl AllocZeroed for TestStruct {}

    let instance = TestStruct::alloc_zeroed_boxed().unwrap();
    assert_eq!(instance.a, 0);
    assert_eq!(instance.b, 0);
    for &byte in &instance.c {
        assert_eq!(byte, 0);
    }
}

#[test]
fn miri_test_buffer_allocation() {
    let mut buffer = [0u8; 1024];

    // Allocate multiple objects in the same buffer
    if let Ok(int_ref) = u32::alloc_zeroed(&mut buffer[..32]) {
        *int_ref = 42;
        assert_eq!(*int_ref, 42);
    }

    if let Ok(float_ref) = f64::alloc_zeroed(&mut buffer[32..64]) {
        *float_ref = std::f64::consts::PI;
        assert_eq!(*float_ref, std::f64::consts::PI);
    }
}
