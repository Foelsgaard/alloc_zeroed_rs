use alloc_zeroed::{AllocZeroed, alloc_zeroed};

#[test]
fn miri_test_primitive() {
    let value = alloc_zeroed::<u32>().unwrap();
    assert_eq!(*value, 0);
}

#[test]
fn miri_test_array() {
    let array = alloc_zeroed::<[u32; 100]>().unwrap();
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

    let instance = alloc_zeroed::<TestStruct>().unwrap();
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
    if let Some(int_ref) = u32::alloc_zeroed(&mut buffer[..32]) {
        *int_ref = 42;
        assert_eq!(*int_ref, 42);
    }

    if let Some(float_ref) = f64::alloc_zeroed(&mut buffer[32..64]) {
        *float_ref = std::f64::consts::PI;
        assert_eq!(*float_ref, std::f64::consts::PI);
    }
}
