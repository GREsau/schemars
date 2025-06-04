#![forbid(unsafe_code)]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

mod ast;
mod attr;
mod bound;
mod idents;
mod name;
mod schema_exprs;

use ast::*;
use idents::GENERATOR;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

#[doc = "Derive macro for `JsonSchema` trait."]
#[cfg_attr(not(doctest), doc = include_str!("../deriving.md"), doc = include_str!("../attributes.md"))]
#[proc_macro_derive(JsonSchema, attributes(schemars, serde, validate, garde))]
pub fn derive_json_schema_wrapper(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    derive_json_schema(input, false)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(JsonSchema_repr, attributes(schemars, serde))]
pub fn derive_json_schema_repr_wrapper(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    derive_json_schema(input, true)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_json_schema(mut input: syn::DeriveInput, repr: bool) -> syn::Result<TokenStream> {
    attr::process_serde_attrs(&mut input)?;

    let mut cont = Container::from_ast(&input)?;
    let relevant_type_params = bound::add_trait_bounds(&mut cont);

    let crate_alias = cont.attrs.crate_name.as_ref().map(|path| {
        quote_spanned! {path.span()=>
            use #path as schemars;
        }
    });

    let type_name = &cont.ident;
    let (impl_generics, ty_generics, where_clause) = cont.generics.split_for_impl();

    if let Some(transparent_field) = cont.transparent_field() {
        // If any schemars attributes for setting metadata (e.g. description) are present, then
        // it's not fully transparent, so use the normal `schema_exprs::expr_for_container`
        // implementation (which always treats the struct as a newtype if it has `transparent`)
        if cont.attrs.common.is_default() && transparent_field.attrs.is_default() {
            let (ty, type_def) = schema_exprs::type_for_field_schema(transparent_field);
            return Ok(quote! {
                const _: () = {
                    #crate_alias
                    #type_def

                    #[automatically_derived]
                    impl #impl_generics schemars::JsonSchema for #type_name #ty_generics #where_clause {
                        fn inline_schema() -> bool {
                            <#ty as schemars::JsonSchema>::inline_schema()
                        }

                        fn schema_name() -> schemars::_private::alloc::borrow::Cow<'static, str> {
                            <#ty as schemars::JsonSchema>::schema_name()
                        }

                        fn schema_id() -> schemars::_private::alloc::borrow::Cow<'static, str> {
                            <#ty as schemars::JsonSchema>::schema_id()
                        }

                        fn json_schema(#GENERATOR: &mut schemars::SchemaGenerator) -> schemars::Schema {
                            <#ty as schemars::JsonSchema>::json_schema(#GENERATOR)
                        }

                        fn _schemars_private_non_optional_json_schema(#GENERATOR: &mut schemars::SchemaGenerator) -> schemars::Schema {
                            <#ty as schemars::JsonSchema>::_schemars_private_non_optional_json_schema(#GENERATOR)
                        }

                        fn _schemars_private_is_option() -> bool {
                            <#ty as schemars::JsonSchema>::_schemars_private_is_option()
                        }
                    };
                };
            });
        }
    }

    let (schema_name, schema_id) = name::get_schema_name_and_id_exprs(&cont, relevant_type_params);

    let schema_expr = if repr {
        schema_exprs::expr_for_repr(&cont)?
    } else {
        schema_exprs::expr_for_container(&cont)
    };

    let inline = cont.attrs.inline;

    Ok(quote! {
        const _: () = {
            #crate_alias

            #[automatically_derived]
            #[allow(unused_braces)]
            impl #impl_generics schemars::JsonSchema for #type_name #ty_generics #where_clause {
                fn schema_name() -> schemars::_private::alloc::borrow::Cow<'static, str> {
                    #schema_name
                }

                fn schema_id() -> schemars::_private::alloc::borrow::Cow<'static, str> {
                    #schema_id
                }

                fn json_schema(#GENERATOR: &mut schemars::SchemaGenerator) -> schemars::Schema {
                    #schema_expr
                }

                fn inline_schema() -> bool {
                    #inline
                }
            };
        };
    })
}
