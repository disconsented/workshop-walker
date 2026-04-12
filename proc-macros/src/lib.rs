use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, Type, parse::Parser, parse_macro_input};

struct DualTypeAttr {
    internal_ty: Type,
    external_ty: Option<Type>,
    to_external: Option<Expr>,
    to_internal: Option<Expr>,
}

impl syn::parse::Parse for DualTypeAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let internal_ty: Type = input.parse()?;
        let mut external_ty = None;
        let mut to_external = None;
        let mut to_internal = None;

        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            // If the next token is an identifier followed by '=', it's a keyword argument
            if !(input.peek(syn::Ident) && input.peek2(syn::Token![=])) {
                external_ty = Some(input.parse()?);
                if input.peek(syn::Token![,]) {
                    input.parse::<syn::Token![,]>()?;
                }
            }
        }

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;
            let expr: Expr = input.parse()?;
            if ident == "to_external" {
                to_external = Some(expr);
            } else if ident == "to_internal" {
                to_internal = Some(expr);
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "expected `to_external` or `to_internal`",
                ));
            }
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(DualTypeAttr {
            internal_ty,
            external_ty,
            to_external,
            to_internal,
        })
    }
}

#[proc_macro_attribute]
pub fn dual_struct(attr_ts: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);
    let original_ident = input.ident.clone();
    let original_attrs = input.attrs.clone();

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

    let mut errors = Vec::new();

    for field in fields.iter_mut() {
        let field_ident = &field.ident;
        let mut dual_type = None;
        let original_ty = field.ty.clone();

        // Check for #[dual_type(InternalType, ExternalType, ...)] or
        // #[dual_type(Type, ...)]
        field.attrs.retain(|attr| {
            if attr.path().is_ident("dual_type") {
                let parse_res = attr.parse_args::<DualTypeAttr>();
                match parse_res {
                    Ok(dual_attr) => {
                        let internal_ty = dual_attr.internal_ty;
                        let external_ty =
                            dual_attr.external_ty.unwrap_or_else(|| original_ty.clone());
                        dual_type = Some((
                            internal_ty,
                            external_ty,
                            dual_attr.to_external,
                            dual_attr.to_internal,
                        ));
                    }
                    Err(e) => {
                        errors.push(e);
                    }
                }
                false // remove the attribute
            } else {
                true
            }
        });

        // Use a temporary field without the removed attribute
        let mut internal_field = field.clone();
        let mut external_field = field.clone();

        if let Some((ref internal_ty, ref external_ty, ref to_external, ref to_internal)) =
            dual_type
        {
            internal_field.ty = internal_ty.clone();
            external_field.ty = external_ty.clone();
            internal_fields.push(quote! { #internal_field });
            external_fields.push(quote! { #external_field });

            let to_ext_expr = if let Some(func) = to_external {
                quote! { #func(item.#field_ident)? }
            } else {
                quote! { item.#field_ident.try_into().map_err(|e| {
                    let err_str = format!("{:?}", e);
                    surrealdb_types::Error::thrown(err_str)
                })? }
            };
            from_internal_to_external.push(quote! { #field_ident: #to_ext_expr });

            let to_int_expr = if let Some(func) = to_internal {
                quote! { #func(item.#field_ident) }
            } else {
                quote! { item.#field_ident.into() }
            };
            from_external_to_internal.push(quote! { #field_ident: #to_int_expr });
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

    let internal_struct_doc = format!(
        "Internal version of [`{}`].\n\n```rust\nstruct {} {{\n    {}\n}}\n```",
        original_ident,
        internal_ident,
        internal_fields
            .iter()
            .map(|f| {
                let s = quote!(#f).to_string();
                // Replace #[doc = r" ..."] with /// ...
                // This is a naive replacement but should work for most cases in docs
                s.replace("# [doc = r\"", "///")
                    .replace("# [doc = \"", "///")
                    .replace("\"]", "")
                    .replace("\"] ", "")
            })
            .collect::<Vec<_>>()
            .join(",\n    ")
    );
    let external_struct_doc = format!(
        "External version of [`{}`].\n\n```rust\nstruct {} {{\n    {}\n}}\n```",
        original_ident,
        external_ident,
        external_fields
            .iter()
            .map(|f| {
                let s = quote!(#f).to_string();
                s.replace("# [doc = r\"", "///")
                    .replace("# [doc = \"", "///")
                    .replace("\"]", "")
                    .replace("\"] ", "")
            })
            .collect::<Vec<_>>()
            .join(",\n    ")
    );

    if !errors.is_empty() {
        let compile_errors = errors.iter().map(|e| e.to_compile_error());
        return TokenStream::from(quote! {
            #(#compile_errors)*
        });
    }

    let expanded = quote! {
        #(#original_attrs)*
        #[doc = #internal_struct_doc]
        #[derive(#(#internal_derives),*)]
        #[serde(rename = #original_name_str)]
        pub struct #internal_ident {
            #(#internal_fields),*
        }

        #(#original_attrs)*
        #[doc = #external_struct_doc]
        #[derive(#(#external_derives),*)]
        #[serde(rename = #original_name_str)]
        pub struct #external_ident {
            #(#external_fields),*
        }

        impl TryFrom<#internal_ident> for #external_ident {
            type Error = surrealdb_types::Error;
            fn try_from(item: #internal_ident) -> Result<Self, Self::Error> {
                Ok(Self {
                    #(#from_internal_to_external),*
                })
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
