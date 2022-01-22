#![forbid(unsafe_code)]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

mod ast;
mod attr;
mod metadata;
mod regex_syntax;
mod schema_exprs;

use ast::*;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

#[proc_macro_derive(JsonSchema, attributes(schemars, serde, validate))]
pub fn derive_json_schema_wrapper(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    derive_json_schema(input, false)
        .unwrap_or_else(compile_error)
        .into()
}

#[proc_macro_derive(JsonSchema_repr, attributes(schemars, serde))]
pub fn derive_json_schema_repr_wrapper(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    derive_json_schema(input, true)
        .unwrap_or_else(compile_error)
        .into()
}

fn derive_json_schema(
    mut input: syn::DeriveInput,
    repr: bool,
) -> Result<TokenStream, Vec<syn::Error>> {
    add_trait_bounds(&mut input.generics);

    attr::process_serde_attrs(&mut input)?;

    let cont = Container::from_ast(&input)?;
    let crate_alias = cont.attrs.crate_name.as_ref().map(|path| {
        quote_spanned! {path.span()=>
            use #path as schemars;
        }
    });

    let type_name = &cont.ident;
    let (impl_generics, ty_generics, where_clause) = cont.generics.split_for_impl();

    if let Some(transparent_field) = cont.transparent_field() {
        let (ty, type_def) = schema_exprs::type_for_field_schema(transparent_field);
        return Ok(quote! {
            const _: () = {
                #crate_alias
                #type_def

                #[automatically_derived]
                impl #impl_generics schemars::JsonSchema for #type_name #ty_generics #where_clause {
                    fn is_referenceable() -> bool {
                        <#ty as schemars::JsonSchema>::is_referenceable()
                    }

                    fn schema_name() -> std::string::String {
                        <#ty as schemars::JsonSchema>::schema_name()
                    }

                    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
                        <#ty as schemars::JsonSchema>::json_schema(gen)
                    }

                    fn _schemars_private_non_optional_json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
                        <#ty as schemars::JsonSchema>::_schemars_private_non_optional_json_schema(gen)
                    }

                    fn _schemars_private_is_option() -> bool {
                        <#ty as schemars::JsonSchema>::_schemars_private_is_option()
                    }
                };
            };
        });
    }

    let mut schema_base_name = cont.name();
    let schema_is_renamed = *type_name != schema_base_name;

    if !schema_is_renamed {
        if let Some(path) = cont.serde_attrs.remote() {
            if let Some(segment) = path.segments.last() {
                schema_base_name = segment.ident.to_string();
            }
        }
    }

    let type_params: Vec<_> = cont.generics.type_params().map(|ty| &ty.ident).collect();
    let schema_name = if type_params.is_empty() {
        quote! {
            #schema_base_name.to_owned()
        }
    } else if schema_is_renamed {
        let mut schema_name_fmt = schema_base_name;
        for tp in &type_params {
            schema_name_fmt.push_str(&format!("{{{}:.0}}", tp));
        }
        quote! {
            format!(#schema_name_fmt #(,#type_params=#type_params::schema_name())*)
        }
    } else {
        let mut schema_name_fmt = schema_base_name;
        schema_name_fmt.push_str("_for_{}");
        schema_name_fmt.push_str(&"_and_{}".repeat(type_params.len() - 1));
        quote! {
            format!(#schema_name_fmt #(,#type_params::schema_name())*)
        }
    };

    let schema_expr = if repr {
        schema_exprs::expr_for_repr(&cont).map_err(|e| vec![e])?
    } else {
        schema_exprs::expr_for_container(&cont)
    };

    Ok(quote! {
        const _: () = {
            #crate_alias

            #[automatically_derived]
            #[allow(unused_braces)]
            impl #impl_generics schemars::JsonSchema for #type_name #ty_generics #where_clause {
                fn schema_name() -> std::string::String {
                    #schema_name
                }

                fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
                    #schema_expr
                }
            };
        };
    })
}

fn add_trait_bounds(generics: &mut syn::Generics) {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(schemars::JsonSchema));
        }
    }
}

fn compile_error(errors: Vec<syn::Error>) -> TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote! {
        #(#compile_errors)*
    }
}
