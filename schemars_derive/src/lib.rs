#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use serde_derive_internals::ast::{Container, Data, Field, Style, Variant};
use serde_derive_internals::{Ctxt, Derive};
use syn::spanned::Spanned;
use syn::{DeriveInput, GenericParam, Generics};

#[proc_macro_derive(MakeSchema, attributes(schemars, serde))]
pub fn derive_make_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    // TODO is mutating the input really the best way to do this?
    add_trait_bounds(&mut input.generics);
    let ctxt = Ctxt::new();
    let cont = Container::from_ast(&ctxt, &input, Derive::Deserialize);
    if let Err(e) = ctxt.check() {
        return compile_error(input.span(), e).into();
    }

    let name = cont.ident;
    let (impl_generics, ty_generics, where_clause) = cont.generics.split_for_impl();

    let schema = match cont.data {
        Data::Struct(Style::Struct, ref fields) => schema_for_struct(fields),
        Data::Enum(ref variants) => schema_for_enum(variants),
        _ => unimplemented!("work in progress!"),
    };

    let impl_block = quote! {
        #[automatically_derived]
        impl #impl_generics schemars::MakeSchema for #name #ty_generics #where_clause {
            fn make_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::Schema {
                #schema
            }
        };
    };
    proc_macro::TokenStream::from(impl_block)
}

fn add_trait_bounds(generics: &mut Generics) {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(schemars::MakeSchema));
        }
    }
}

fn wrap_schema_fields(schema_contents: TokenStream) -> TokenStream {
    quote! {
        schemars::SchemaObject {
            #schema_contents
            ..Default::default()
        }
        .into()
    }
}

fn compile_error(span: Span, message: String) -> TokenStream {
    quote_spanned! {span=>
        compile_error!(#message);
    }
}

fn is_unit_variant(v: &&Variant) -> bool {
    match v.style {
        Style::Unit => true,
        _ => false,
    }
}

fn schema_for_enum(variants: &[Variant]) -> TokenStream {
    // TODO handle untagged or adjacently tagged enums
    let (unit_variants, complex_variants): (Vec<_>, Vec<_>) =
        variants.into_iter().partition(is_unit_variant);
    let unit_count = unit_variants.len();

    let unit_names = unit_variants
        .into_iter()
        .map(|v| v.attrs.name().deserialize_name());
    let unit_schema = wrap_schema_fields(quote! {
        enum_values: Some(vec![#(#unit_names.into()),*]),
    });

    if complex_variants.is_empty() {
        return unit_schema;
    }

    let mut schemas = Vec::new();
    if unit_count > 0 {
        schemas.push(unit_schema);
    }

    schemas.extend(complex_variants.into_iter().map(|variant| {
        let name = variant.attrs.name().deserialize_name();
        let sub_schema = match variant.style {
            Style::Newtype => {
                let f = &variant.fields[0];
                let ty = f.ty;
                quote_spanned! {f.original.span()=>
                    gen.subschema_for::<#ty>()
                }
            }
            Style::Tuple => {
                let types = variant.fields.iter().map(|f| f.ty);
                quote! {
                    gen.subschema_for::<(#(#types),*)>()
                }
            }
            Style::Struct => schema_for_struct(&variant.fields),
            Style::Unit => unreachable!("Unit variants already filtered out"),
        };

        wrap_schema_fields(quote! {
            properties: {
                let mut props = std::collections::BTreeMap::new();
                props.insert(#name.to_owned(), #sub_schema);
                props
            },
        })
    }));

    wrap_schema_fields(quote! {
        any_of: Some(vec![#(#schemas),*]),
    })
}

fn schema_for_struct(fields: &[Field]) -> TokenStream {
    let recurse = fields.into_iter().map(|f| {
        let name = f.attrs.name().deserialize_name();
        let ty = f.ty;
        quote_spanned! {f.original.span()=>
                props.insert(#name.to_owned(), gen.subschema_for::<#ty>());
        }
    });
    wrap_schema_fields(quote! {
        properties: {
            let mut props = std::collections::BTreeMap::new();
            #(#recurse)*
            props
        },
    })
}
