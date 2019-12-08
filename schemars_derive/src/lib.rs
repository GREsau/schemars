#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

mod doc_attrs;
mod metadata;
mod preprocess;

use metadata::*;
use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::ToTokens;
use serde_derive_internals::ast::{Container, Data, Field, Style, Variant};
use serde_derive_internals::attr::{self, Default as SerdeDefault, TagType};
use serde_derive_internals::{Ctxt, Derive};
use syn::parse::{self, Parse};
use syn::spanned::Spanned;

#[proc_macro_derive(JsonSchema, attributes(schemars, serde, doc))]
pub fn derive_json_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as syn::DeriveInput);

    preprocess::add_trait_bounds(&mut input.generics);
    if let Err(e) = preprocess::process_serde_attrs(&mut input) {
        return compile_error(e).into();
    }

    let ctxt = Ctxt::new();
    let cont = Container::from_ast(&ctxt, &input, Derive::Deserialize);
    if let Err(e) = ctxt.check() {
        return compile_error(e).into();
    }
    let cont = cont.expect("from_ast set no errors on Ctxt, so should have returned Some");

    let schema_expr = match cont.data {
        Data::Struct(Style::Unit, _) => schema_for_unit_struct(),
        Data::Struct(Style::Newtype, ref fields) => schema_for_newtype_struct(&fields[0]),
        Data::Struct(Style::Tuple, ref fields) => schema_for_tuple_struct(fields),
        Data::Struct(Style::Struct, ref fields) => schema_for_struct(fields, &cont.attrs),
        Data::Enum(ref variants) => schema_for_enum(variants, &cont.attrs),
    };
    let schema_expr = set_metadata_on_schema_from_docs(schema_expr, &cont.original.attrs);

    let type_name = cont.ident;
    let type_params: Vec<_> = cont.generics.type_params().map(|ty| &ty.ident).collect();

    let schema_base_name = cont.attrs.name().deserialize_name();
    let schema_name = if type_params.is_empty() {
        quote! {
            #schema_base_name.to_owned()
        }
    } else if type_name == schema_base_name {
        let mut schema_name_fmt = schema_base_name;
        schema_name_fmt.push_str("_for_{}");
        schema_name_fmt.push_str(&"_and_{}".repeat(type_params.len() - 1));
        quote! {
            format!(#schema_name_fmt #(,#type_params::schema_name())*)
        }
    } else {
        let mut schema_name_fmt = schema_base_name;
        for tp in &type_params {
            schema_name_fmt.push_str(&format!("{{{}:.0}}", tp));
        }
        quote! {
            format!(#schema_name_fmt #(,#type_params=#type_params::schema_name())*)
        }
    };

    let (impl_generics, ty_generics, where_clause) = cont.generics.split_for_impl();

    let impl_block = quote! {
        #[automatically_derived]
        impl #impl_generics schemars::JsonSchema for #type_name #ty_generics #where_clause {
            fn schema_name() -> String {
                #schema_name
            }

            fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
                #schema_expr
            }
        };
    };
    proc_macro::TokenStream::from(impl_block)
}

fn wrap_schema_fields(schema_contents: TokenStream) -> TokenStream {
    quote! {
        schemars::schema::Schema::Object(
            schemars::schema::SchemaObject {
            #schema_contents
            ..Default::default()
        })
    }
}

fn compile_error(errors: Vec<syn::Error>) -> TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote! {
        #(#compile_errors)*
    }
}

fn is_unit_variant(v: &Variant) -> bool {
    match v.style {
        Style::Unit => true,
        _ => false,
    }
}

fn schema_for_enum(variants: &[Variant], cattrs: &attr::Container) -> TokenStream {
    let variants = variants.iter().filter(|v| !v.attrs.skip_deserializing());
    match cattrs.tag() {
        TagType::External => schema_for_external_tagged_enum(variants, cattrs),
        TagType::None => schema_for_untagged_enum(variants, cattrs),
        TagType::Internal { tag } => schema_for_internal_tagged_enum(variants, cattrs, tag),
        TagType::Adjacent { .. } => unimplemented!("Adjacent tagged enums not yet supported."),
    }
}

fn schema_for_external_tagged_enum<'a>(
    variants: impl Iterator<Item = &'a Variant<'a>>,
    cattrs: &attr::Container,
) -> TokenStream {
    let (unit_variants, complex_variants): (Vec<_>, Vec<_>) =
        variants.partition(|v| is_unit_variant(v));
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
        let schema_expr = wrap_schema_fields(quote! {
            instance_type: Some(schemars::schema::InstanceType::Object.into()),
            object: Some(Box::new(schemars::schema::ObjectValidation {
                properties: {
                    let mut props = schemars::Map::new();
                    props.insert(#name.to_owned(), #sub_schema);
                    props
                },
                required: {
                    let mut required = schemars::Set::new();
                    required.insert(#name.to_owned());
                    required
                },
                ..Default::default()
            })),
        });
        set_metadata_on_schema_from_docs(schema_expr, &variant.original.attrs)
    }));

    wrap_schema_fields(quote! {
        subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
            any_of: Some(vec![#(#schemas),*]),
            ..Default::default()
        })),
    })
}

fn schema_for_internal_tagged_enum<'a>(
    variants: impl Iterator<Item = &'a Variant<'a>>,
    cattrs: &attr::Container,
    tag_name: &str,
) -> TokenStream {
    let schemas = variants.map(|variant| {
        let name = variant.attrs.name().deserialize_name();
        let type_schema = wrap_schema_fields(quote! {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            enum_values: Some(vec![#name.into()]),
        });

        let tag_schema = wrap_schema_fields(quote! {
            instance_type: Some(schemars::schema::InstanceType::Object.into()),
            object: Some(Box::new(schemars::schema::ObjectValidation {
                properties: {
                    let mut props = schemars::Map::new();
                    props.insert(#tag_name.to_owned(), #type_schema);
                    props
                },
                required: {
                    let mut required = schemars::Set::new();
                    required.insert(#tag_name.to_owned());
                    required
                },
                ..Default::default()
            })),
        });
        let tag_schema = set_metadata_on_schema_from_docs(tag_schema, &variant.original.attrs);

        let variant_schema = match variant.style {
            Style::Unit => return tag_schema,
            Style::Newtype => {
                let field = &variant.fields[0];
                let ty = get_json_schema_type(field);
                quote_spanned! {field.original.span()=>
                    <#ty>::json_schema(gen)
                }
            }
            Style::Struct => schema_for_struct(&variant.fields, cattrs),
            Style::Tuple => unreachable!("Internal tagged enum tuple variants will have caused serde_derive_internals to output a compile error already."),
        };
        quote! {
            #tag_schema.flatten(#variant_schema)
        }
    });

    wrap_schema_fields(quote! {
        subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
            any_of: Some(vec![#(#schemas),*]),
            ..Default::default()
        })),
    })
}

fn schema_for_untagged_enum<'a>(
    variants: impl Iterator<Item = &'a Variant<'a>>,
    cattrs: &attr::Container,
) -> TokenStream {
    let schemas = variants.map(|variant| {
        let schema_expr = schema_for_untagged_enum_variant(variant, cattrs);
        set_metadata_on_schema_from_docs(schema_expr, &variant.original.attrs)
    });

    wrap_schema_fields(quote! {
        subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
            any_of: Some(vec![#(#schemas),*]),
            ..Default::default()
        })),
    })
}

fn schema_for_untagged_enum_variant(variant: &Variant, cattrs: &attr::Container) -> TokenStream {
    match variant.style {
        Style::Unit => schema_for_unit_struct(),
        Style::Newtype => schema_for_newtype_struct(&variant.fields[0]),
        Style::Tuple => schema_for_tuple_struct(&variant.fields),
        Style::Struct => schema_for_struct(&variant.fields, cattrs),
    }
}

fn schema_for_unit_struct() -> TokenStream {
    quote! {
        gen.subschema_for::<()>()
    }
}

fn schema_for_newtype_struct(field: &Field) -> TokenStream {
    let ty = get_json_schema_type(field);
    quote_spanned! {field.original.span()=>
        gen.subschema_for::<#ty>()
    }
}

fn schema_for_tuple_struct(fields: &[Field]) -> TokenStream {
    let types = fields
        .iter()
        .filter(|f| !f.attrs.skip_deserializing())
        .map(get_json_schema_type);
    quote! {
        gen.subschema_for::<(#(#types),*)>()
    }
}

fn schema_for_struct(fields: &[Field], cattrs: &attr::Container) -> TokenStream {
    let (flat, nested): (Vec<_>, Vec<_>) = fields
        .iter()
        .filter(|f| !f.attrs.skip_deserializing() || !f.attrs.skip_serializing())
        .partition(|f| f.attrs.flatten());

    let set_container_default = match cattrs.default() {
        SerdeDefault::None => None,
        SerdeDefault::Default => Some(quote!(let cdefault = Self::default();)),
        SerdeDefault::Path(path) => Some(quote!(let cdefault = #path();)),
    };

    let mut required = Vec::new();
    let recurse = nested.iter().map(|field| {
        let name = field.attrs.name().deserialize_name();
        let ty = field.ty;

        let default = match field.attrs.default() {
            SerdeDefault::None if set_container_default.is_some() => {
                let field_ident = field
                    .original
                    .ident
                    .as_ref()
                    .expect("This is not a tuple struct, so field should be named");
                Some(quote!(cdefault.#field_ident))
            }
            SerdeDefault::None => None,
            SerdeDefault::Default => Some(quote!(<#ty>::default())),
            SerdeDefault::Path(path) => Some(quote!(#path())),
        }
        .map(|d| match field.attrs.serialize_with() {
            Some(ser_with) => quote! {
                {
                    struct _SchemarsDefaultSerialize<T>(T);

                    impl serde::Serialize for _SchemarsDefaultSerialize<#ty>
                    {
                        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where
                            S: serde::Serializer
                        {
                            #ser_with(&self.0, serializer)
                        }
                    }

                    _SchemarsDefaultSerialize(#d)
                }
            },
            None => d,
        });

        if default.is_none() {
            required.push(name.clone());
        }

        let ty = get_json_schema_type(field);
        let span = field.original.span();
        let schema_expr = quote_spanned! {span=>
            gen.subschema_for::<#ty>()
        };

        let metadata = SchemaMetadata {
            read_only: field.attrs.skip_deserializing(),
            write_only: field.attrs.skip_serializing(),
            default,
            ..get_metadata_from_docs(&field.original.attrs)
        };
        let schema_expr = set_metadata_on_schema(schema_expr, &metadata);

        quote_spanned! {span=>
            props.insert(#name.to_owned(), #schema_expr);
        }
    });

    let schema = wrap_schema_fields(quote! {
        instance_type: Some(schemars::schema::InstanceType::Object.into()),
        object: Some(Box::new(schemars::schema::ObjectValidation {
            properties: {
                let mut props = schemars::Map::new();
                #(#recurse)*
                props
            },
            required: {
                let mut required = schemars::Set::new();
                #(required.insert(#required.to_owned());)*
                required
            },
            ..Default::default()
        })),
    });

    let flattens = flat.iter().map(|field| {
        let ty = get_json_schema_type(field);
        quote_spanned! {field.original.span()=>
            .flatten(<#ty>::json_schema_optional(gen))
        }
    });

    quote! {
        {
            #set_container_default
            #schema #(#flattens)*
        }
    }
}

fn get_json_schema_type(field: &Field) -> Box<dyn ToTokens> {
    // TODO support [schemars(schema_with= "...")] or equivalent
    match field
        .original
        .attrs
        .iter()
        .filter(|at| match at.path.get_ident() {
            // FIXME this is relying on order of attributes (schemars before serde) from preprocess.rs
            Some(i) => i == "schemars" || i == "serde",
            None => false,
        })
        .filter_map(get_with_from_attr)
        .next()
    {
        Some(with) => match parse_lit_str::<syn::ExprPath>(&with) {
            Ok(expr_path) => Box::new(expr_path),
            Err(e) => Box::new(compile_error(vec![e])),
        },
        None => Box::new(field.ty.clone()),
    }
}

fn get_with_from_attr(attr: &syn::Attribute) -> Option<syn::LitStr> {
    use syn::*;
    let nested_metas = match attr.parse_meta() {
        Ok(Meta::List(meta)) => meta.nested,
        _ => return None,
    };
    for nm in nested_metas {
        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
            path,
            lit: Lit::Str(with),
            ..
        })) = nm
        {
            if path.is_ident("with") {
                return Some(with);
            }
        }
    }
    None
}

fn parse_lit_str<T>(s: &syn::LitStr) -> parse::Result<T>
where
    T: Parse,
{
    let tokens = spanned_tokens(s)?;
    syn::parse2(tokens)
}

fn spanned_tokens(s: &syn::LitStr) -> parse::Result<TokenStream> {
    let stream = syn::parse_str(&s.value())?;
    Ok(respan_token_stream(stream, s.span()))
}

fn respan_token_stream(stream: TokenStream, span: Span) -> TokenStream {
    stream
        .into_iter()
        .map(|token| respan_token_tree(token, span))
        .collect()
}

fn respan_token_tree(mut token: TokenTree, span: Span) -> TokenTree {
    if let TokenTree::Group(g) = &mut token {
        *g = Group::new(g.delimiter(), respan_token_stream(g.stream(), span));
    }
    token.set_span(span);
    token
}
