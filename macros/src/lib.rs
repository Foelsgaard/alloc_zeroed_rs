// macros/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data};

#[proc_macro_derive(AllocZeroed)]
pub fn derive_alloc_zeroed(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    
    // Check if this is a struct
    let fields = match input.data {
        Data::Struct(data_struct) => data_struct.fields,
        _ => {
            return syn::Error::new(
                name.span(),
                "AllocZeroed can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };
    
    // Generate field assignments for the implementation
    let field_inits = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote! {
            #field_name: core::mem::zeroed()
        }
    });
    
    let expanded = quote! {
        // SAFETY: This macro ensures all fields can be safely zero-initialized
        unsafe impl AllocZeroed for #name {
            fn alloc_zeroed(mem: &mut [u8]) -> Option<&mut Self> {
                use core::mem;
                
                let size = mem::size_of::<Self>();
                let align = mem::align_of::<Self>();
                let len = mem.len();
                
                let mem_ptr = mem.as_mut_ptr();
                let offset = mem_ptr.align_offset(align);
                
                if offset == usize::MAX || size > len - offset {
                    return None;
                }
                
                let ptr = unsafe { mem_ptr.add(offset) as *mut Self };
                unsafe {
                    // Initialize the struct with zeroed fields
                    ptr.write(Self {
                        #(#field_inits,)*
                    });
                    ptr.as_mut()
                }
            }
        }
    };
    
    TokenStream::from(expanded)
}
