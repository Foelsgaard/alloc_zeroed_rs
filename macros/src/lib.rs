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
        unsafe impl #impl_generics AllocZeroed for #name #ty_generics #where_clause {}
    };
    
    TokenStream::from(expanded)
}
