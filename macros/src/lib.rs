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
    
    // Extract field types for the where clause
    let field_types = fields.iter().map(|field| &field.ty);
    
    // Generate field assignments for the implementation
    let field_inits = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote! {
            #field_name: core::mem::zeroed()
        }
    });
    
    // Clone generics before modifying to avoid borrowing issues
    let mut generics = input.generics.clone();
    let where_clause = generics.make_where_clause();
    for ty in field_types {
        where_clause.predicates.push(syn::parse_quote! { #ty: AllocZeroed });
    }
    
    // Now split the original generics (not the modified one)
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    
    let expanded = quote! {
        // SAFETY: This macro ensures all fields can be safely zero-initialized
        // by requiring that all field types implement AllocZeroed
        unsafe impl #impl_generics AllocZeroed for #name #ty_generics #where_clause {
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
                
                // SAFETY: We've checked that the offset is valid and there's enough space
                let ptr = unsafe { mem_ptr.add(offset) } as *mut Self;
                
                // SAFETY: We've ensured the pointer is properly aligned and there's enough space
                // All fields implement AllocZeroed, so zero-initialization is safe
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
