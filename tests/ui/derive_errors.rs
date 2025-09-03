use alloc_zeroed::AllocZeroed;

// This should fail to compile because String can't be zero-initialized
#[derive(AllocZeroed)]
struct InvalidStruct {
    value: String, // String has a non-zero invalid state
}

// This should fail to compile because the enum can't be zero-initialized
#[derive(AllocZeroed)]
enum InvalidEnum {
    A,
    B,
}
