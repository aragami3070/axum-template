use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Ident, Type, parse_macro_input};

fn raise_error(name: &Ident, describe: &str) -> proc_macro2::TokenStream {
    syn::Error::new_spanned(name, describe).to_compile_error()
}

#[proc_macro_derive(NewTypeDeref)]
pub fn new_type_deref(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let Data::Struct(ref data_struct) = input.data {
        match data_struct.fields {
            syn::Fields::Unnamed(ref fields) => fields,
            syn::Fields::Unit => return raise_error(name, "Unit struct not supported").into(),
            syn::Fields::Named(_) => {
                return raise_error(name, "Named fields not supported").into();
            }
        }
    } else {
        return raise_error(name, "Only struct supported").into();
    };

    let target_type: Type = if let Some(field) = fields.unnamed.first() {
        field.ty.clone()
    } else {
        return raise_error(name, "No fields found").into();
    };

    let gen_deref = quote! {
        impl std::ops::Deref for #name {
            type Target = #target_type;
            fn deref(&self) -> &Self::Target{
                &self.0
            }
        }
    };

    gen_deref.into()
}
