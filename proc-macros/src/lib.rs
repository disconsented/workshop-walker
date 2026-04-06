use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, DeriveInput, Type};

#[proc_macro_attribute]
pub fn dual_struct(attr_ts: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);
    let original_ident = input.ident.clone();

    let fields = match &mut input.data {
        syn::Data::Struct(s) => &mut s.fields,
        _ => {
            return TokenStream::from(quote! {
                compile_error!("dual_struct can only be applied to structs");
            });
        }
    };

    let mut internal_fields = Vec::new();
    let mut external_fields = Vec::new();
    let mut from_internal_to_external = Vec::new();
    let mut from_external_to_internal = Vec::new();

    let mut internal_derives = vec![];
    let mut external_derives = vec![];

    // Parse the attribute input: #[dual_struct(derive(A, B), internal_derive(C),
    // external_derive(D))]
    let mut shared_derives = Vec::new();

    {
        let attr_parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("derive") {
                meta.parse_nested_meta(|meta| {
                    shared_derives.push(meta.path);
                    Ok(())
                })
            } else {
                Err(meta.error("unsupported attribute"))
            }
        });

        if !attr_ts.is_empty() {
            if let Err(e) = attr_parser.parse(attr_ts) {
                return TokenStream::from(e.to_compile_error());
            }
        }
    }

    for d in shared_derives {
        internal_derives.push(quote! { #d });
        external_derives.push(quote! { #d });
    }

    // Requirement 6: Specific derives
    internal_derives.push(quote! { surrealdb_types::SurrealValue });
    external_derives.push(quote! { salvo::prelude::ToSchema });

    for field in fields.iter_mut() {
        let field_ident = &field.ident;
        let mut dual_type = None;

        // Check for #[dual_type(InternalType, ExternalType)]
        field.attrs.retain(|attr| {
            if attr.path().is_ident("dual_type") {
                let result = attr.parse_nested_meta(|meta| {
                    let internal_type: Type = meta.input.parse()?;
                    meta.input.parse::<syn::Token![,]>()?;
                    let external_type: Type = meta.input.parse()?;
                    dual_type = Some((internal_type, external_type));
                    Ok(())
                });
                if result.is_err() {
                    // Fallback or error? For now just ignore if it fails to
                    // parse as expected
                }
                false // remove the attribute
            } else {
                true
            }
        });

        if let Some((internal_ty, external_ty)) = dual_type {
            internal_fields.push(quote! { #field #field_ident: #internal_ty });
            external_fields.push(quote! { #field #field_ident: #external_ty });
            from_internal_to_external.push(quote! { #field_ident: item.#field_ident.into() });
            from_external_to_internal.push(quote! { #field_ident: item.#field_ident.into() });
        } else {
            internal_fields.push(quote! { #field });
            external_fields.push(quote! { #field });
            from_internal_to_external.push(quote! { #field_ident: item.#field_ident });
            from_external_to_internal.push(quote! { #field_ident: item.#field_ident });
        }
    }

    let internal_ident = syn::Ident::new(
        &format!("Internal{}", original_ident),
        original_ident.span(),
    );
    let external_ident = syn::Ident::new(
        &format!("External{}", original_ident),
        original_ident.span(),
    );
    let original_name_str = original_ident.to_string();

    let expanded = quote! {
        #[derive(#(#internal_derives),*)]
        #[serde(rename = #original_name_str)]
        pub struct #internal_ident {
            #(#internal_fields),*
        }

        #[derive(#(#external_derives),*)]
        #[serde(rename = #original_name_str)]
        pub struct #external_ident {
            #(#external_fields),*
        }

        impl From<#internal_ident> for #external_ident {
            fn from(item: #internal_ident) -> Self {
                Self {
                    #(#from_internal_to_external),*
                }
            }
        }

        impl From<#external_ident> for #internal_ident {
            fn from(item: #external_ident) -> Self {
                Self {
                    #(#from_external_to_internal),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
