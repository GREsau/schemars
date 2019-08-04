#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use serde_derive_internals::ast::{Container, Data, Field, Style, Variant};
use serde_derive_internals::{Ctxt, Derive};
use syn::spanned::Spanned;
use syn::DeriveInput;

#[proc_macro_derive(MakeSchema, attributes(schemars, serde))]
pub fn derive_make_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ctxt = Ctxt::new();
    let cont = Container::from_ast(&ctxt, &input, Derive::Deserialize);
    if let Err(e) = ctxt.check() {
        return compile_error(input.span(), e).into();
    }

    let name = cont.ident;
    let (impl_generics, ty_generics, where_clause) = cont.generics.split_for_impl();

    let fn_contents = match cont.data {
        Data::Struct(Style::Struct, ref fields) => struct_implementation(fields),
        Data::Enum(ref variants) => enum_implementation(variants),
        _ => unimplemented!("work in progress!"),
    };

    let impl_block = quote! {
        #[automatically_derived]
        impl #impl_generics schemars::make_schema::MakeSchema for #name #ty_generics #where_clause {
            fn make_schema(gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
                let mut o = schemars::SchemaObject {
                    ..Default::default()
                };
                #fn_contents
                o.into()
            }
        };
    };
    proc_macro::TokenStream::from(impl_block)
}

fn compile_error(span: Span, message: String) -> TokenStream {
    quote_spanned! {span=>
        compile_error!(#message);
    }
}

fn is_unit_variant(v: &Variant) -> bool {
    match v.style {
        Style::Unit => true,
        _ => false,
    }
}

fn enum_implementation(variants: &[Variant]) -> TokenStream {
    if variants.iter().all(is_unit_variant) {
        let names = variants
            .into_iter()
            .map(|v| v.attrs.name().deserialize_name());
        return quote! {
            o.enum_values = Some(vec![#(#names.into()),*]);
        };
    }
    unimplemented!("work in progress!")
}

fn struct_implementation(fields: &[Field]) -> TokenStream {
    let recurse = fields.into_iter().map(|f| {
        let name = f.attrs.name().deserialize_name();
        let ty = f.ty;
        quote_spanned! {f.original.span()=>
            o.properties.insert(#name.to_owned(), gen.subschema_for::<#ty>());
        }
    });
    quote! {
        #(#recurse)*
    }
}
