use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, GenericParam, Type, TypePath};

#[proc_macro_derive(ConvertId)]
pub fn derive_convert_id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    // Find the generic parameter (assuming there's only one relevant one for now,
    // or just handle all of them) For simplicity, we assume the first generic
    // parameter is the ID type.
    let type_param = generics.params.iter().find_map(|p| {
        if let GenericParam::Type(t) = p {
            Some(&t.ident)
        } else {
            None
        }
    });

    let type_param = match type_param {
        Some(tp) => tp,
        None => {
            return TokenStream::from(quote! {
                compile_error!("ConvertId derive requires at least one generic type parameter");
            });
        }
    };

    let fields = match &input.data {
        syn::Data::Struct(s) => &s.fields,
        _ => {
            return TokenStream::from(quote! {
                compile_error!("ConvertId can only be derived for structs");
            });
        }
    };

    let from_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;

        if is_type_param(field_type, type_param) {
            quote! { #field_name: val.#field_name.into() }
        } else {
            quote! { #field_name: val.#field_name }
        }
    });

    let try_from_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;

        if is_type_param(field_type, type_param) {
            quote! { #field_name: val.#field_name.try_into()? }
        } else {
            quote! { #field_name: val.#field_name }
        }
    });

    let expanded = quote! {
        impl From<#name<External>> for #name<Internal> {
            fn from(val: #name<External>) -> Self {
                Self {
                    #(#from_fields),*
                }
            }
        }

        impl TryFrom<#name<Internal>> for #name<External> {
            type Error = surrealdb_types::Error;
            fn try_from(val: #name<Internal>) -> Result<Self, Self::Error> {
                Ok(Self {
                    #(#try_from_fields),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_type_param(ty: &Type, param: &syn::Ident) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.first() {
            return segment.ident == *param;
        }
    }
    false
}
