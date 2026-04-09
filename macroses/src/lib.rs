use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Type, parse_macro_input};

#[proc_macro_derive(NewTypeDeref)]
pub fn new_type_deref(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let Data::Struct(ref data_struct) = input.data {
        match data_struct.fields {
            syn::Fields::Unnamed(ref fields) => fields,
            syn::Fields::Unit => {
                return syn::Error::new_spanned(&name, "Unit struct not supported")
                    .to_compile_error()
                    .into();
            }
            syn::Fields::Named(_) => {
                return syn::Error::new_spanned(&name, "Named fields not supported")
                    .to_compile_error()
                    .into();
            }
        }
    } else {
        return syn::Error::new_spanned(&name, "Only struct supported")
            .to_compile_error()
            .into();
    };

    let target_type: Type = if let Some(field) = fields.unnamed.first() {
        field.ty.clone()
    } else {
        return syn::Error::new_spanned(&name, "No fields found")
            .to_compile_error()
            .into();
    };

    let gen_deref = quote! {
        impl Deref for #name {
            type Target = #target_type;
            fn deref(&self) -> &Self::Target{
                &self.0
            }
        }
    };

    gen_deref.into()
}
