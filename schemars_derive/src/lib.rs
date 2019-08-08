#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;

mod preprocess;

use proc_macro2::{Span, TokenStream};
use serde_derive_internals::ast::{Container, Data, Field, Style, Variant};
use serde_derive_internals::attr::{self, Default as SerdeDefault, EnumTag};
use serde_derive_internals::{Ctxt, Derive};
use syn::spanned::Spanned;
use syn::DeriveInput;

#[proc_macro_derive(MakeSchema, attributes(schemars, serde))]
pub fn derive_make_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    preprocess::add_trait_bounds(&mut input.generics);
    preprocess::rename_schemars_attrs(&mut input);

    let ctxt = Ctxt::new();
    let cont = Container::from_ast(&ctxt, &input, Derive::Deserialize);
    if let Err(e) = ctxt.check() {
        return compile_error(input.span(), e).into();
    }

    let schema = match cont.data {
        Data::Struct(Style::Struct, ref fields) => schema_for_struct(fields, &cont.attrs),
        Data::Enum(ref variants) => schema_for_enum(variants, &cont.attrs),
        _ => unimplemented!("work in progress!"),
    };

    let type_name = cont.ident;
    let type_params: Vec<_> = cont.generics.type_params().map(|ty| &ty.ident).collect();

    let schema_base_name = cont.attrs.name().deserialize_name();
    let schema_name = if type_params.is_empty() {
        quote! {
            #schema_base_name.to_owned()
        }
    } else if schema_base_name == type_name.to_string() {
        let mut schema_name_fmt = schema_base_name;
        schema_name_fmt.push_str("_For_{}");
        schema_name_fmt.push_str(&"_And_{}".repeat(type_params.len() - 1));
        quote! {
            format!(#schema_name_fmt #(,#type_params::schema_name())*)
        }
    } else {
        let mut schema_name_fmt = schema_base_name;
        for tp in &type_params {
            schema_name_fmt.push_str(&format!("{{{}:.0}}", tp));
        }
        let fmt_param_names = &type_params;
        let type_params = &type_params;
        quote! {
            format!(#schema_name_fmt #(,#fmt_param_names=#type_params::schema_name())*)
        }
    };

    let (impl_generics, ty_generics, where_clause) = cont.generics.split_for_impl();

    let impl_block = quote! {
        #[automatically_derived]
        impl #impl_generics schemars::MakeSchema for #type_name #ty_generics #where_clause {
            fn schema_name() -> String {
                #schema_name
            }

            fn make_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::Result {
                #schema
            }
        };
    };
    proc_macro::TokenStream::from(impl_block)
}

fn wrap_schema_fields(schema_contents: TokenStream) -> TokenStream {
    quote! {
        Ok(schemars::schema::Schema::Object(
            schemars::schema::SchemaObject {
            #schema_contents
            ..Default::default()
        }))
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

fn schema_for_enum(variants: &[Variant], cattrs: &attr::Container) -> TokenStream {
    match cattrs.tag() {
        EnumTag::External => schema_for_external_tagged_enum(variants, cattrs),
        EnumTag::None => schema_for_untagged_enum(variants, cattrs),
        _ => unimplemented!("Adjacent/internal tagged enums not yet supported."),
    }
}

fn schema_for_external_tagged_enum(variants: &[Variant], cattrs: &attr::Container) -> TokenStream {
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
        let sub_schema = schema_for_untagged_enum_variant(variant, cattrs);
        wrap_schema_fields(quote! {
            instance_type: Some(schemars::schema::InstanceType::Object.into()),
            properties: {
                let mut props = std::collections::BTreeMap::new();
                props.insert(#name.to_owned(), #sub_schema);
                props
            },
            required: vec![#name.to_owned()],
        })
    }));

    wrap_schema_fields(quote! {
        any_of: Some(vec![#(#schemas),*]),
    })
}

fn schema_for_untagged_enum(variants: &[Variant], cattrs: &attr::Container) -> TokenStream {
    let schemas = variants
        .into_iter()
        .map(|v| schema_for_untagged_enum_variant(v, cattrs));

    wrap_schema_fields(quote! {
        any_of: Some(vec![#(#schemas),*]),
    })
}

fn schema_for_untagged_enum_variant(variant: &Variant, cattrs: &attr::Container) -> TokenStream {
    match variant.style {
        Style::Unit => quote! {
            gen.subschema_for::<()>()?
        },
        Style::Newtype => {
            let f = &variant.fields[0];
            let ty = f.ty;
            quote_spanned! {f.original.span()=>
                gen.subschema_for::<#ty>()?
            }
        }
        Style::Tuple => {
            let types = variant.fields.iter().map(|f| f.ty);
            quote! {
                gen.subschema_for::<(#(#types),*)>()?
            }
        }
        Style::Struct => schema_for_struct(&variant.fields, cattrs),
    }
}

fn schema_for_struct(fields: &[Field], cattrs: &attr::Container) -> TokenStream {
    let (nested, flat): (Vec<_>, Vec<_>) = fields.iter().partition(|f| !f.attrs.flatten());
    let container_has_default = has_default(cattrs.default());
    let mut required = Vec::new();
    let recurse = nested.iter().map(|f| {
        let name = f.attrs.name().deserialize_name();
        if !container_has_default && !has_default(f.attrs.default()) {
            required.push(name.clone());
        }
        let ty = f.ty;
        quote_spanned! {f.original.span()=>
            props.insert(#name.to_owned(), gen.subschema_for::<#ty>()?);
        }
    });

    let schema = wrap_schema_fields(quote! {
        instance_type: Some(schemars::schema::InstanceType::Object.into()),
        properties: {
            let mut props = std::collections::BTreeMap::new();
            #(#recurse)*
            props
        },
        required: {
            let mut required = std::collections::BTreeSet::new();
            #(required.insert(#required.to_owned());)*
            required
        },
    });

    let flattens = flat.iter().map(|f| {
        let ty = f.ty;
        quote_spanned! {f.original.span()=>
            ?.flatten(<#ty>::make_schema(gen)?)
        }
    });

    quote! {
        #schema #(#flattens)*
    }
}

fn has_default(d: &SerdeDefault) -> bool {
    match d {
        SerdeDefault::None => false,
        _ => true,
    }
}
