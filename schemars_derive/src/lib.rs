#![forbid(unsafe_code)]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

mod ast;
mod attr;
mod metadata;
mod schema_exprs;

use ast::*;
use proc_macro2::TokenStream;

#[proc_macro_derive(JsonSchema, attributes(schemars, serde))]
pub fn derive_json_schema_wrapper(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    derive_json_schema(input).into()
}

fn derive_json_schema(mut input: syn::DeriveInput) -> TokenStream {
    if let Err(e) = attr::process_serde_attrs(&mut input) {
        return compile_error(&e);
    }

    let cont = match Container::from_ast(&input) {
        Ok(c) => c,
        Err(e) => return compile_error(&e),
    };

    let default_crate_name: syn::Path = parse_quote!(schemars);
    let crate_name = cont
        .attrs
        .crate_name
        .as_ref()
        .unwrap_or(&default_crate_name);

    let mut gen = cont.generics.clone();

    add_trait_bounds(&crate_name, &mut gen);

    let type_name = &cont.ident;
    let (impl_generics, ty_generics, where_clause) = gen.split_for_impl();

    if let Some(transparent_field) = cont.transparent_field() {
        let (ty, type_def) = schema_exprs::type_for_schema(crate_name, transparent_field, 0);
        return quote! {
            #[automatically_derived]
            impl #impl_generics #crate_name::JsonSchema for #type_name #ty_generics #where_clause {
                #type_def

                fn is_referenceable() -> bool {
                    <#ty as #crate_name::JsonSchema>::is_referenceable()
                }

                fn schema_name() -> std::string::String {
                    <#ty as #crate_name::JsonSchema>::schema_name()
                }

                fn json_schema(gen: &mut #crate_name::gen::SchemaGenerator) -> #crate_name::schema::Schema {
                    <#ty as #crate_name::JsonSchema>::json_schema(gen)
                }

                fn json_schema_for_flatten(gen: &mut #crate_name::gen::SchemaGenerator) -> #crate_name::schema::Schema {
                    <#ty as #crate_name::JsonSchema>::json_schema_for_flatten(gen)
                }

                fn add_schema_as_property(
                    gen: &mut #crate_name::gen::SchemaGenerator,
                    parent: &mut #crate_name::schema::SchemaObject,
                    name: String,
                    metadata: Option<#crate_name::schema::Metadata>,
                    required: bool,
                ) {
                    <#ty as #crate_name::JsonSchema>::add_schema_as_property(gen, parent, name, metadata, required)
                }
            };
        };
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

    let schema_expr = schema_exprs::expr_for_container(&cont);

    quote! {
        #[automatically_derived]
        #[allow(unused_braces)]
        impl #impl_generics #crate_name::JsonSchema for #type_name #ty_generics #where_clause {
            fn schema_name() -> std::string::String {
                #schema_name
            }

            fn json_schema(gen: &mut #crate_name::gen::SchemaGenerator) -> #crate_name::schema::Schema {
                #schema_expr
            }
        };
    }
}

fn add_trait_bounds(crate_name: &syn::Path, generics: &mut syn::Generics) {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(#crate_name::JsonSchema));
        }
    }
}

fn compile_error<'a>(errors: impl IntoIterator<Item = &'a syn::Error>) -> TokenStream {
    let compile_errors = errors.into_iter().map(syn::Error::to_compile_error);
    quote! {
        #(#compile_errors)*
    }
}
