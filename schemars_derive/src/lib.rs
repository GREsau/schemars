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
    add_trait_bounds(&mut input.generics);

    if let Err(e) = attr::process_serde_attrs(&mut input) {
        return compile_error(&e);
    }

    let cont = match Container::from_ast(&input) {
        Ok(c) => c,
        Err(e) => return compile_error(&e),
    };

    let schema_expr = schema_exprs::expr_for_container(&cont);

    let type_name = &cont.ident;
    let type_params: Vec<_> = cont.generics.type_params().map(|ty| &ty.ident).collect();

    let mut schema_base_name = cont.name();
    let schema_is_renamed = *type_name != schema_base_name;

    if !schema_is_renamed {
        if let Some(path) = cont.serde_attrs.remote() {
            if let Some(segment) = path.segments.last() {
                schema_base_name = segment.ident.to_string();
            }
        }
    }

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

    let (impl_generics, ty_generics, where_clause) = cont.generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics schemars::JsonSchema for #type_name #ty_generics #where_clause {
            fn schema_name() -> std::string::String {
                #schema_name
            }

            fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
                #schema_expr
            }
        };
    }
}

fn add_trait_bounds(generics: &mut syn::Generics) {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(schemars::JsonSchema));
        }
    }
}

fn compile_error<'a>(errors: impl IntoIterator<Item = &'a syn::Error>) -> TokenStream {
    let compile_errors = errors.into_iter().map(syn::Error::to_compile_error);
    quote! {
        #(#compile_errors)*
    }
}
