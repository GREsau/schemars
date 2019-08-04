#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;

use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields};

#[proc_macro_derive(MakeSchema, attributes(schemars, serde))]
pub fn derive_make_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fn_contents = match input.data {
        Data::Struct(data) => struct_implementation(&data.fields),
        _ => unimplemented!("Only structs work so far!"),
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

fn struct_implementation(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(ref fields) => {
            let recurse = fields.named.iter().map(|f| {
                let name = (&f.ident).as_ref().map(ToString::to_string);
                let ty = &f.ty;
                quote_spanned! {f.span()=>
                    o.properties.insert(#name.to_owned(), gen.subschema_for::<#ty>());
                }
            });
            quote! {
                #(#recurse)*
            }
        }
        _ => unimplemented!("Only named fields work so far!"),
    }
}
