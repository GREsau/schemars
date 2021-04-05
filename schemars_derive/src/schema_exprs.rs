use crate::{ast::*, attr::WithAttr, metadata::SchemaMetadata};
use proc_macro2::{Span, TokenStream};
use serde_derive_internals::ast::Style;
use serde_derive_internals::attr::{self as serde_attr, Default as SerdeDefault, TagType};
use syn::spanned::Spanned;

pub fn expr_for_container(cont: &Container) -> TokenStream {
    let schema_expr = match &cont.data {
        Data::Struct(Style::Unit, _) => expr_for_unit_struct(),
        Data::Struct(Style::Newtype, fields) => expr_for_newtype_struct(&fields[0]),
        Data::Struct(Style::Tuple, fields) => expr_for_tuple_struct(fields),
        Data::Struct(Style::Struct, fields) => expr_for_struct(
            fields,
            cont.serde_attrs.default(),
            cont.serde_attrs.deny_unknown_fields(),
        ),
        Data::Enum(variants) => expr_for_enum(variants, &cont.serde_attrs),
    };

    let doc_metadata = SchemaMetadata::from_attrs(&cont.attrs);
    doc_metadata.apply_to_schema(schema_expr)
}

pub fn expr_for_repr(cont: &Container) -> Result<TokenStream, syn::Error> {
    let repr_type = cont.attrs.repr.as_ref().ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "JsonSchema_repr: missing #[repr(...)] attribute",
        )
    })?;

    let variants = match &cont.data {
        Data::Enum(variants) => variants,
        _ => return Err(syn::Error::new(Span::call_site(), "oh no!")),
    };

    if let Some(non_unit_error) = variants.iter().find_map(|v| match v.style {
        Style::Unit => None,
        _ => Some(syn::Error::new(
            v.original.span(),
            "JsonSchema_repr: must be a unit variant",
        )),
    }) {
        return Err(non_unit_error);
    };

    let enum_ident = &cont.ident;
    let variant_idents = variants.iter().map(|v| &v.ident);

    let schema_expr = schema_object(quote! {
        instance_type: Some(schemars::schema::InstanceType::Integer.into()),
        enum_values: Some(vec![#((#enum_ident::#variant_idents as #repr_type).into()),*]),
    });

    let doc_metadata = SchemaMetadata::from_attrs(&cont.attrs);
    Ok(doc_metadata.apply_to_schema(schema_expr))
}

fn expr_for_field(field: &Field, allow_ref: bool) -> TokenStream {
    let (ty, type_def) = type_for_schema(field, 0);
    let span = field.original.span();
    let gen = quote!(gen);

    if allow_ref {
        quote_spanned! {span=>
            {
                #type_def
                #gen.subschema_for::<#ty>()
            }
        }
    } else {
        quote_spanned! {span=>
            {
                #type_def
                <#ty as schemars::JsonSchema>::json_schema(#gen)
            }
        }
    }
}

pub fn type_for_schema(field: &Field, local_id: usize) -> (syn::Type, Option<TokenStream>) {
    match &field.attrs.with {
        None => (field.ty.to_owned(), None),
        Some(WithAttr::Type(ty)) => (ty.to_owned(), None),
        Some(WithAttr::Function(fun)) => {
            let ty_name = format_ident!("_SchemarsSchemaWithFunction{}", local_id);
            let fn_name = fun.segments.last().unwrap().ident.to_string();

            let type_def = quote_spanned! {fun.span()=>
                struct #ty_name;

                impl schemars::JsonSchema for #ty_name {
                    fn is_referenceable() -> bool {
                        false
                    }

                    fn schema_name() -> std::string::String {
                        #fn_name.to_string()
                    }

                    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
                        #fun(gen)
                    }
                }
            };

            (parse_quote!(#ty_name), Some(type_def))
        }
    }
}

fn expr_for_enum(variants: &[Variant], cattrs: &serde_attr::Container) -> TokenStream {
    let deny_unknown_fields = cattrs.deny_unknown_fields();
    let variants = variants
        .iter()
        .filter(|v| !v.serde_attrs.skip_deserializing());

    match cattrs.tag() {
        TagType::External => expr_for_external_tagged_enum(variants, deny_unknown_fields),
        TagType::None => expr_for_untagged_enum(variants, deny_unknown_fields),
        TagType::Internal { tag } => {
            expr_for_internal_tagged_enum(variants, tag, deny_unknown_fields)
        }
        TagType::Adjacent { tag, content } => {
            expr_for_adjacent_tagged_enum(variants, tag, content, deny_unknown_fields)
        }
    }
}

fn expr_for_external_tagged_enum<'a>(
    variants: impl Iterator<Item = &'a Variant<'a>>,
    deny_unknown_fields: bool,
) -> TokenStream {
    let (unit_variants, complex_variants): (Vec<_>, Vec<_>) =
        variants.partition(|v| v.is_unit() && v.attrs.with.is_none());

    let unit_names = unit_variants.iter().map(|v| v.name());
    let unit_schema = schema_object(quote! {
        instance_type: Some(schemars::schema::InstanceType::String.into()),
        enum_values: Some(vec![#(#unit_names.into()),*]),
    });

    if complex_variants.is_empty() {
        return unit_schema;
    }

    let mut schemas = Vec::new();
    if !unit_variants.is_empty() {
        schemas.push(unit_schema);
    }

    schemas.extend(complex_variants.into_iter().map(|variant| {
        let name = variant.name();
        let sub_schema = expr_for_untagged_enum_variant(variant, deny_unknown_fields);

        let schema_expr = schema_object(quote! {
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
                additional_properties: Some(Box::new(false.into())),
                ..Default::default()
            })),
        });
        let doc_metadata = SchemaMetadata::from_attrs(&variant.attrs);
        doc_metadata.apply_to_schema(schema_expr)
    }));

    schema_object(quote! {
        subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
            any_of: Some(vec![#(#schemas),*]),
            ..Default::default()
        })),
    })
}

fn expr_for_internal_tagged_enum<'a>(
    variants: impl Iterator<Item = &'a Variant<'a>>,
    tag_name: &str,
    deny_unknown_fields: bool,
) -> TokenStream {
    let variant_schemas = variants.map(|variant| {
        let name = variant.name();
        let type_schema = schema_object(quote! {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            enum_values: Some(vec![#name.into()]),
        });

        let tag_schema = schema_object(quote! {
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
        let doc_metadata = SchemaMetadata::from_attrs(&variant.attrs);
        let tag_schema = doc_metadata.apply_to_schema(tag_schema);

        match expr_for_untagged_enum_variant_for_flatten(&variant, deny_unknown_fields) {
            Some(variant_schema) => quote! {
                #tag_schema.flatten(#variant_schema)
            },
            None => tag_schema,
        }
    });

    schema_object(quote! {
        subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
            any_of: Some(vec![#(#variant_schemas),*]),
            ..Default::default()
        })),
    })
}

fn expr_for_untagged_enum<'a>(
    variants: impl Iterator<Item = &'a Variant<'a>>,
    deny_unknown_fields: bool,
) -> TokenStream {
    let schemas = variants.map(|variant| {
        let schema_expr = expr_for_untagged_enum_variant(variant, deny_unknown_fields);
        let doc_metadata = SchemaMetadata::from_attrs(&variant.attrs);
        doc_metadata.apply_to_schema(schema_expr)
    });

    schema_object(quote! {
        subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
            any_of: Some(vec![#(#schemas),*]),
            ..Default::default()
        })),
    })
}

fn expr_for_adjacent_tagged_enum<'a>(
    variants: impl Iterator<Item = &'a Variant<'a>>,
    tag_name: &str,
    content_name: &str,
    deny_unknown_fields: bool,
) -> TokenStream {
    let schemas = variants.map(|variant| {
        let content_schema = if variant.is_unit() && variant.attrs.with.is_none() {
            None
        } else {
            Some(expr_for_untagged_enum_variant(variant, deny_unknown_fields))
        };

        let (add_content_to_props, add_content_to_required) = content_schema
            .map(|content_schema| {
                (
                    quote!(props.insert(#content_name.to_owned(), #content_schema);),
                    quote!(required.insert(#content_name.to_owned());),
                )
            })
            .unwrap_or_default();

        let name = variant.name();
        let tag_schema = schema_object(quote! {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            enum_values: Some(vec![#name.into()]),
        });

        let set_additional_properties = if deny_unknown_fields {
            quote! {
                additional_properties: Some(Box::new(false.into())),
            }
        } else {
            TokenStream::new()
        };

        let outer_schema = schema_object(quote! {
            instance_type: Some(schemars::schema::InstanceType::Object.into()),
            object: Some(Box::new(schemars::schema::ObjectValidation {
                properties: {
                    let mut props = schemars::Map::new();
                    props.insert(#tag_name.to_owned(), #tag_schema);
                    #add_content_to_props
                    props
                },
                required: {
                    let mut required = schemars::Set::new();
                    required.insert(#tag_name.to_owned());
                    #add_content_to_required
                    required
                },
                #set_additional_properties
                ..Default::default()
            })),
        });

        let doc_metadata = SchemaMetadata::from_attrs(&variant.attrs);
        doc_metadata.apply_to_schema(outer_schema)
    });

    schema_object(quote! {
        subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
            any_of: Some(vec![#(#schemas),*]),
            ..Default::default()
        })),
    })
}

fn expr_for_untagged_enum_variant(variant: &Variant, deny_unknown_fields: bool) -> TokenStream {
    if let Some(WithAttr::Type(with)) = &variant.attrs.with {
        let gen = quote!(gen);
        return quote_spanned! {variant.original.span()=>
            #gen.subschema_for::<#with>()
        };
    }

    match variant.style {
        Style::Unit => expr_for_unit_struct(),
        Style::Newtype => expr_for_field(&variant.fields[0], true),
        Style::Tuple => expr_for_tuple_struct(&variant.fields),
        Style::Struct => expr_for_struct(&variant.fields, &SerdeDefault::None, deny_unknown_fields),
    }
}

fn expr_for_untagged_enum_variant_for_flatten(
    variant: &Variant,
    deny_unknown_fields: bool,
) -> Option<TokenStream> {
    if let Some(WithAttr::Type(with)) = &variant.attrs.with {
        let gen = quote!(gen);
        return Some(quote_spanned! {variant.original.span()=>
            <#with as schemars::JsonSchema>::json_schema(#gen)
        });
    }

    Some(match variant.style {
        Style::Unit => return None,
        Style::Newtype => expr_for_field(&variant.fields[0], false),
        Style::Tuple => expr_for_tuple_struct(&variant.fields),
        Style::Struct => expr_for_struct(&variant.fields, &SerdeDefault::None, deny_unknown_fields),
    })
}

fn expr_for_unit_struct() -> TokenStream {
    quote! {
        gen.subschema_for::<()>()
    }
}

fn expr_for_newtype_struct(field: &Field) -> TokenStream {
    expr_for_field(field, true)
}

fn expr_for_tuple_struct(fields: &[Field]) -> TokenStream {
    let (types, type_defs): (Vec<_>, Vec<_>) = fields
        .iter()
        .filter(|f| !f.serde_attrs.skip_deserializing())
        .enumerate()
        .map(|(i, f)| type_for_schema(f, i))
        .unzip();
    quote! {
        {
            #(#type_defs)*
            gen.subschema_for::<(#(#types),*)>()
        }
    }
}

fn expr_for_struct(
    fields: &[Field],
    default: &SerdeDefault,
    deny_unknown_fields: bool,
) -> TokenStream {
    let (flattened_fields, property_fields): (Vec<_>, Vec<_>) = fields
        .iter()
        .filter(|f| !f.serde_attrs.skip_deserializing() || !f.serde_attrs.skip_serializing())
        .partition(|f| f.serde_attrs.flatten());

    let set_container_default = match default {
        SerdeDefault::None => None,
        SerdeDefault::Default => Some(quote!(let container_default = Self::default();)),
        SerdeDefault::Path(path) => Some(quote!(let container_default = #path();)),
    };

    let mut type_defs = Vec::new();

    let properties: Vec<_> = property_fields
        .into_iter()
        .map(|field| {
            let name = field.name();
            let default = field_default_expr(field, set_container_default.is_some());

            let required = match default {
                Some(_) => quote!(false),
                None => quote!(true),
            };

            let metadata = &SchemaMetadata {
                read_only: field.serde_attrs.skip_deserializing(),
                write_only: field.serde_attrs.skip_serializing(),
                default,
                ..SchemaMetadata::from_attrs(&field.attrs)
            };

            let (ty, type_def) = type_for_schema(field, type_defs.len());
            if let Some(type_def) = type_def {
                type_defs.push(type_def);
            }

            let args = quote!(gen, &mut schema_object, #name.to_owned(), #metadata, #required);

            quote_spanned! {ty.span()=>
                <#ty as schemars::JsonSchema>::add_schema_as_property(#args);
            }
        })
        .collect();

    let flattens: Vec<_> = flattened_fields
        .into_iter()
        .map(|field| {
            let (ty, type_def) = type_for_schema(field, type_defs.len());
            if let Some(type_def) = type_def {
                type_defs.push(type_def);
            }

            let gen = quote!(gen);
            quote_spanned! {ty.span()=>
                .flatten(<#ty as schemars::JsonSchema>::json_schema_for_flatten(#gen))
            }
        })
        .collect();

    let set_additional_properties = if deny_unknown_fields {
        quote! {
            schema_object.object().additional_properties = Some(Box::new(false.into()));
        }
    } else {
        TokenStream::new()
    };
    quote! {
        {
            #(#type_defs)*
            #set_container_default
            let mut schema_object = schemars::schema::SchemaObject {
                instance_type: Some(schemars::schema::InstanceType::Object.into()),
                ..Default::default()
            };
            #set_additional_properties
            #(#properties)*
            schemars::schema::Schema::Object(schema_object)
            #(#flattens)*
        }
    }
}

fn field_default_expr(field: &Field, container_has_default: bool) -> Option<TokenStream> {
    let field_default = field.serde_attrs.default();
    if field.serde_attrs.skip_serializing() || (field_default.is_none() && !container_has_default) {
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

    let default_expr = match field.serde_attrs.skip_serializing_if() {
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

    Some(if let Some(ser_with) = field.serde_attrs.serialize_with() {
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

fn schema_object(properties: TokenStream) -> TokenStream {
    quote! {
        schemars::schema::Schema::Object(
            schemars::schema::SchemaObject {
            #properties
            ..Default::default()
        })
    }
}
