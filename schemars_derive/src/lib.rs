#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

mod attr;
mod metadata;

use metadata::*;
use proc_macro2::TokenStream;
use quote::ToTokens;
use serde_derive_internals::ast::{Container, Data, Field, Style, Variant};
use serde_derive_internals::attr::{self as serde_attr, Default as SerdeDefault, TagType};
use serde_derive_internals::{Ctxt, Derive};
use syn::spanned::Spanned;

#[proc_macro_derive(JsonSchema, attributes(schemars, serde))]
pub fn derive_json_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as syn::DeriveInput);

    add_trait_bounds(&mut input.generics);
    if let Err(e) = attr::process_serde_attrs(&mut input) {
        return compile_error(&e).into();
    }

    let ctxt = Ctxt::new();
    let cont = Container::from_ast(&ctxt, &input, Derive::Deserialize);
    if let Err(e) = ctxt.check() {
        return compile_error(&e).into();
    }
    let cont = cont.expect("from_ast set no errors on Ctxt, so should have returned Some");

    let schema_expr = match cont.data {
        Data::Struct(Style::Unit, _) => schema_for_unit_struct(),
        Data::Struct(Style::Newtype, ref fields) => schema_for_newtype_struct(&fields[0]),
        Data::Struct(Style::Tuple, ref fields) => schema_for_tuple_struct(fields),
        Data::Struct(Style::Struct, ref fields) => schema_for_struct(fields, Some(&cont.attrs)),
        Data::Enum(ref variants) => schema_for_enum(variants, &cont.attrs),
    };
    let doc_metadata = SchemaMetadata::from_doc_attrs(&cont.original.attrs);
    let schema_expr = doc_metadata.apply_to_schema(schema_expr);

    let type_name = cont.ident;
    let type_params: Vec<_> = cont.generics.type_params().map(|ty| &ty.ident).collect();

    let mut schema_base_name = cont.attrs.name().deserialize_name();
    let schema_is_renamed = type_name != schema_base_name;

    if !schema_is_renamed {
        if let Some(path) = cont.attrs.remote() {
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

    let impl_block = quote! {
        #[automatically_derived]
        impl #impl_generics schemars::JsonSchema for #type_name #ty_generics #where_clause {
            fn schema_name() -> std::string::String {
                #schema_name
            }

            fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
                #schema_expr
            }
        };
    };
    proc_macro::TokenStream::from(impl_block)
}

fn add_trait_bounds(generics: &mut syn::Generics) {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(schemars::JsonSchema));
        }
    }
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

fn compile_error<'a>(errors: impl IntoIterator<Item = &'a syn::Error>) -> TokenStream {
    let compile_errors = errors.into_iter().map(syn::Error::to_compile_error);
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

fn schema_for_enum(variants: &[Variant], cattrs: &serde_attr::Container) -> TokenStream {
    let variants = variants.iter().filter(|v| !v.attrs.skip_deserializing());
    match cattrs.tag() {
        TagType::External => schema_for_external_tagged_enum(variants),
        TagType::None => schema_for_untagged_enum(variants),
        TagType::Internal { tag } => schema_for_internal_tagged_enum(variants, tag),
        TagType::Adjacent { .. } => unimplemented!("Adjacent tagged enums not yet supported."),
    }
}

fn schema_for_external_tagged_enum<'a>(
    variants: impl Iterator<Item = &'a Variant<'a>>,
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
        let sub_schema = schema_for_untagged_enum_variant(variant);
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
        let doc_metadata = SchemaMetadata::from_doc_attrs(&variant.original.attrs);
        doc_metadata.apply_to_schema(schema_expr)
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
        let doc_metadata = SchemaMetadata::from_doc_attrs(&variant.original.attrs);
        let tag_schema = doc_metadata.apply_to_schema(tag_schema);

        let variant_schema = match variant.style {
            Style::Unit => return tag_schema,
            Style::Newtype => {
                let field = &variant.fields[0];
                let ty = get_json_schema_type(field);
                quote_spanned! {field.original.span()=>
                    <#ty>::json_schema(gen)
                }
            }
            Style::Struct => schema_for_struct(&variant.fields, None),
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

fn schema_for_untagged_enum<'a>(variants: impl Iterator<Item = &'a Variant<'a>>) -> TokenStream {
    let schemas = variants.map(|variant| {
        let schema_expr = schema_for_untagged_enum_variant(variant);
        let doc_metadata = SchemaMetadata::from_doc_attrs(&variant.original.attrs);
        doc_metadata.apply_to_schema(schema_expr)
    });

    wrap_schema_fields(quote! {
        subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
            any_of: Some(vec![#(#schemas),*]),
            ..Default::default()
        })),
    })
}

fn schema_for_untagged_enum_variant(variant: &Variant) -> TokenStream {
    match variant.style {
        Style::Unit => schema_for_unit_struct(),
        Style::Newtype => schema_for_newtype_struct(&variant.fields[0]),
        Style::Tuple => schema_for_tuple_struct(&variant.fields),
        Style::Struct => schema_for_struct(&variant.fields, None),
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

fn schema_for_struct(fields: &[Field], cattrs: Option<&serde_attr::Container>) -> TokenStream {
    let (flattened_fields, property_fields): (Vec<_>, Vec<_>) = fields
        .iter()
        .filter(|f| !f.attrs.skip_deserializing() || !f.attrs.skip_serializing())
        .partition(|f| f.attrs.flatten());

    let set_container_default = match cattrs.map_or(&SerdeDefault::None, |c| c.default()) {
        SerdeDefault::None => None,
        SerdeDefault::Default => Some(quote!(let container_default = Self::default();)),
        SerdeDefault::Path(path) => Some(quote!(let container_default = #path();)),
    };

    let properties = property_fields.iter().map(|field| {
        let name = field.attrs.name().deserialize_name();
        let default = field_default_expr(field, set_container_default.is_some());

        let required = match default {
            Some(_) => quote!(false),
            None => quote!(true),
        };

        let metadata = &SchemaMetadata {
            read_only: field.attrs.skip_deserializing(),
            write_only: field.attrs.skip_serializing(),
            default,
            ..SchemaMetadata::from_doc_attrs(&field.original.attrs)
        };

        let ty = get_json_schema_type(field);
        let span = field.original.span();

        quote_spanned! {span=>
            <#ty>::add_schema_as_property(gen, &mut schema_object, #name.to_owned(), #metadata, #required);
        }
    });

    let flattens = flattened_fields.iter().map(|field| {
        let ty = get_json_schema_type(field);
        let span = field.original.span();

        quote_spanned! {span=>
            .flatten(<#ty>::json_schema_for_flatten(gen))
        }
    });

    quote! {
        {
            #set_container_default
            let mut schema_object = schemars::schema::SchemaObject {
                instance_type: Some(schemars::schema::InstanceType::Object.into()),
                ..Default::default()
            };
            #(#properties)*
            schemars::schema::Schema::Object(schema_object)
            #(#flattens)*
        }
    }
}

fn field_default_expr(field: &Field, container_has_default: bool) -> Option<TokenStream> {
    let field_default = field.attrs.default();
    if field.attrs.skip_serializing() || (field_default.is_none() && !container_has_default) {
        return None;
    }

    let ty = field.ty;
    let default_expr = match field_default {
        SerdeDefault::None => {
            let member = &field.member;
            quote!(container_default.#member)
        }
        SerdeDefault::Default => quote!(<#ty>::default()),
        SerdeDefault::Path(path) => quote!(#path()),
    };

    let default_expr = match field.attrs.skip_serializing_if() {
        Some(skip_if) => {
            quote! {
                {
                    let default = #default_expr;
                    if #skip_if(&default) {
                        None
                    } else {
                        Some(default)
                    }
                }
            }
        }
        None => quote!(Some(#default_expr)),
    };

    Some(if let Some(ser_with) = field.attrs.serialize_with() {
        quote! {
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

                #default_expr.map(|d| _SchemarsDefaultSerialize(d))
            }
        }
    } else {
        default_expr
    })
}

fn get_json_schema_type(field: &Field) -> Box<dyn ToTokens> {
    // TODO support [schemars(schema_with= "...")] or equivalent
    match attr::get_with_from_attrs(&field.original) {
        None => Box::new(field.ty.clone()),
        Some(Ok(expr_path)) => Box::new(expr_path),
        Some(Err(e)) => Box::new(compile_error(&[e])),
    }
}
