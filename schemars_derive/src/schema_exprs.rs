use std::collections::HashSet;

use crate::{ast::*, attr::WithAttr, metadata::SchemaMetadata};
use proc_macro2::{Span, TokenStream};
use serde_derive_internals::ast::Style;
use serde_derive_internals::attr::{self as serde_attr, Default as SerdeDefault, TagType};
use syn::spanned::Spanned;

pub fn expr_for_container(cont: &Container) -> TokenStream {
    let mut schema_expr = match &cont.data {
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

    cont.attrs.as_metadata().apply_to_schema(&mut schema_expr);
    schema_expr
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

    let mut schema_expr = quote!({
        let mut map = schemars::_serde_json::Map::new();
        map.insert("type".into(), "integer".into());
        map.insert(
            "enum".into(),
            schemars::_serde_json::Value::Array({
                let mut enum_values = schemars::_alloc::vec::Vec::new();
                #(enum_values.push((#enum_ident::#variant_idents as #repr_type).into());)*
                enum_values
            }),
        );
        schemars::Schema::from(map)
    });

    cont.attrs.as_metadata().apply_to_schema(&mut schema_expr);
    Ok(schema_expr)
}

fn expr_for_field(field: &Field, allow_ref: bool) -> TokenStream {
    let (ty, type_def) = type_for_field_schema(field);
    let span = field.original.span();
    let generator = quote!(generator);

    let mut schema_expr = if field.validation_attrs.required() {
        quote_spanned! {span=>
            <#ty as schemars::JsonSchema>::_schemars_private_non_optional_json_schema(#generator)
        }
    } else if allow_ref {
        quote_spanned! {span=>
            #generator.subschema_for::<#ty>()
        }
    } else {
        quote_spanned! {span=>
            <#ty as schemars::JsonSchema>::json_schema(#generator)
        }
    };

    prepend_type_def(type_def, &mut schema_expr);
    field.validation_attrs.apply_to_schema(&mut schema_expr);

    schema_expr
}

pub fn type_for_field_schema(field: &Field) -> (syn::Type, Option<TokenStream>) {
    match &field.attrs.with {
        None => (field.ty.to_owned(), None),
        Some(with_attr) => type_for_schema(with_attr),
    }
}

fn type_for_schema(with_attr: &WithAttr) -> (syn::Type, Option<TokenStream>) {
    match with_attr {
        WithAttr::Type(ty) => (ty.to_owned(), None),
        WithAttr::Function(fun) => {
            let ty_name = syn::Ident::new("_SchemarsSchemaWithFunction", Span::call_site());
            let fn_name = fun.segments.last().unwrap().ident.to_string();

            let type_def = quote_spanned! {fun.span()=>
                struct #ty_name;

                impl schemars::JsonSchema for #ty_name {
                    fn always_inline_schema() -> bool {
                        true
                    }

                    fn schema_name() -> schemars::_alloc::borrow::Cow<'static, str> {
                        schemars::_alloc::borrow::Cow::Borrowed(#fn_name)
                    }

                    fn schema_id() -> schemars::_alloc::borrow::Cow<'static, str> {
                        schemars::_alloc::borrow::Cow::Borrowed(::core::concat!(
                            "_SchemarsSchemaWithFunction/",
                            ::core::module_path!(),
                            "/",
                            #fn_name
                        ))
                    }

                    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
                        #fun(generator)
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
    let mut unique_names = HashSet::<&str>::new();
    let mut count = 0;
    let (unit_variants, complex_variants): (Vec<_>, Vec<_>) = variants
        .inspect(|v| {
            unique_names.insert(v.name());
            count += 1;
        })
        .partition(|v| v.is_unit() && v.attrs.is_default());
    let unit_names = unit_variants.iter().map(|v| v.name());
    let unit_schema = quote!({
        let mut map = schemars::_serde_json::Map::new();
        map.insert("type".into(), "string".into());
        map.insert(
            "enum".into(),
            schemars::_serde_json::Value::Array({
                let mut enum_values = schemars::_alloc::vec::Vec::new();
                #(enum_values.push((#unit_names).into());)*
                enum_values
            }),
        );
        schemars::Schema::from(map)
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

        let mut schema_expr = if variant.is_unit() && variant.attrs.with.is_none() {
            quote! {
                schemars::_private::new_unit_enum_variant(#name)
            }
        } else {
            let sub_schema = expr_for_untagged_enum_variant(variant, deny_unknown_fields);
            quote! {
                schemars::_private::new_externally_tagged_enum_variant(#name, #sub_schema)
            }
        };

        variant
            .attrs
            .as_metadata()
            .apply_to_schema(&mut schema_expr);

        schema_expr
    }));

    variant_subschemas(unique_names.len() == count, schemas)
}

fn expr_for_internal_tagged_enum<'a>(
    variants: impl Iterator<Item = &'a Variant<'a>>,
    tag_name: &str,
    deny_unknown_fields: bool,
) -> TokenStream {
    let mut unique_names = HashSet::new();
    let mut count = 0;
    let variant_schemas = variants
        .map(|variant| {
            unique_names.insert(variant.name());
            count += 1;

            let name = variant.name();

            let mut schema_expr = expr_for_internal_tagged_enum_variant(variant, deny_unknown_fields);
            schema_expr = quote!({
                let mut schema = #schema_expr;
                schemars::_private::apply_internal_enum_variant_tag(&mut schema, #tag_name, #name, #deny_unknown_fields);
                schema
            });
            variant.attrs.as_metadata().apply_to_schema(&mut schema_expr);
            schema_expr
        })
        .collect();

    variant_subschemas(unique_names.len() == count, variant_schemas)
}

fn expr_for_untagged_enum<'a>(
    variants: impl Iterator<Item = &'a Variant<'a>>,
    deny_unknown_fields: bool,
) -> TokenStream {
    let schemas = variants
        .map(|variant| {
            let mut schema_expr = expr_for_untagged_enum_variant(variant, deny_unknown_fields);

            variant
                .attrs
                .as_metadata()
                .apply_to_schema(&mut schema_expr);

            schema_expr
        })
        .collect();

    // Untagged enums can easily have variants whose schemas overlap; rather
    // that checking the exclusivity of each subschema we simply us `any_of`.
    variant_subschemas(false, schemas)
}

fn expr_for_adjacent_tagged_enum<'a>(
    variants: impl Iterator<Item = &'a Variant<'a>>,
    tag_name: &str,
    content_name: &str,
    deny_unknown_fields: bool,
) -> TokenStream {
    let mut unique_names = HashSet::new();
    let mut count = 0;
    let schemas = variants
        .map(|variant| {
            unique_names.insert(variant.name());
            count += 1;

            let content_schema = if variant.is_unit() && variant.attrs.with.is_none() {
                None
            } else {
                Some(expr_for_untagged_enum_variant(variant, deny_unknown_fields))
            };

            let (add_content_to_props, add_content_to_required) = content_schema
                .map(|content_schema| {
                    (
                        quote!(#content_name: (#content_schema),),
                        quote!(#content_name,),
                    )
                })
                .unwrap_or_default();

            let name = variant.name();
            let tag_schema = quote! {
                schemars::json_schema!({
                    "type": "string",
                    "enum": [#name],
                })
            };

            let set_additional_properties = if deny_unknown_fields {
                quote! {
                    "additionalProperties": false,
                }
            } else {
                TokenStream::new()
            };

            let mut outer_schema = quote! {
                schemars::json_schema!({
                    "type": "object",
                    "properties": {
                        #tag_name: (#tag_schema),
                        #add_content_to_props
                    },
                    "required": [
                        #tag_name,
                        #add_content_to_required
                    ],
                    // As we're creating a "wrapper" object, we can honor the
                    // disposition of deny_unknown_fields.
                    #set_additional_properties
                })
            };

            variant
                .attrs
                .as_metadata()
                .apply_to_schema(&mut outer_schema);

            outer_schema
        })
        .collect();

    variant_subschemas(unique_names.len() == count, schemas)
}

/// Callers must determine if all subschemas are mutually exclusive. This can
/// be done for most tagging regimes by checking that all tag names are unique.
fn variant_subschemas(unique: bool, schemas: Vec<TokenStream>) -> TokenStream {
    let keyword = if unique { "oneOf" } else { "anyOf" };
    quote!({
        let mut map = schemars::_serde_json::Map::new();
        map.insert(
            #keyword.into(),
            schemars::_serde_json::Value::Array({
                let mut enum_values = schemars::_alloc::vec::Vec::new();
                #(enum_values.push(#schemas.to_value());)*
                enum_values
            }),
        );
        schemars::Schema::from(map)
    })
}

fn expr_for_untagged_enum_variant(variant: &Variant, deny_unknown_fields: bool) -> TokenStream {
    if let Some(with_attr) = &variant.attrs.with {
        let (ty, type_def) = type_for_schema(with_attr);
        let generator = quote!(generator);
        let mut schema_expr = quote_spanned! {variant.original.span()=>
            #generator.subschema_for::<#ty>()
        };

        prepend_type_def(type_def, &mut schema_expr);
        return schema_expr;
    }

    match variant.style {
        Style::Unit => expr_for_unit_struct(),
        Style::Newtype => expr_for_field(&variant.fields[0], true),
        Style::Tuple => expr_for_tuple_struct(&variant.fields),
        Style::Struct => expr_for_struct(&variant.fields, &SerdeDefault::None, deny_unknown_fields),
    }
}

fn expr_for_internal_tagged_enum_variant(
    variant: &Variant,
    deny_unknown_fields: bool,
) -> TokenStream {
    if let Some(with_attr) = &variant.attrs.with {
        let (ty, type_def) = type_for_schema(with_attr);
        let generator = quote!(generator);
        let mut schema_expr = quote_spanned! {variant.original.span()=>
            <#ty as schemars::JsonSchema>::json_schema(#generator)
        };

        prepend_type_def(type_def, &mut schema_expr);
        return schema_expr;
    }

    match variant.style {
        Style::Unit => expr_for_unit_struct(),
        Style::Newtype => expr_for_field(&variant.fields[0], false),
        Style::Tuple => expr_for_tuple_struct(&variant.fields),
        Style::Struct => expr_for_struct(&variant.fields, &SerdeDefault::None, deny_unknown_fields),
    }
}

fn expr_for_unit_struct() -> TokenStream {
    quote! {
        generator.subschema_for::<()>()
    }
}

fn expr_for_newtype_struct(field: &Field) -> TokenStream {
    expr_for_field(field, true)
}

fn expr_for_tuple_struct(fields: &[Field]) -> TokenStream {
    let fields: Vec<_> = fields
        .iter()
        .filter(|f| !f.serde_attrs.skip_deserializing())
        .map(|f| expr_for_field(f, true))
        .collect();
    let len = fields.len() as u32;

    quote! {
        schemars::json_schema!({
            "type": "array",
            "prefixItems": [#((#fields)),*],
            "minItems": #len,
            "maxItems": #len,
        })
    }
}

fn expr_for_struct(
    fields: &[Field],
    default: &SerdeDefault,
    deny_unknown_fields: bool,
) -> TokenStream {
    let set_container_default = match default {
        SerdeDefault::None => None,
        SerdeDefault::Default => Some(quote!(let container_default = Self::default();)),
        SerdeDefault::Path(path) => Some(quote!(let container_default = #path();)),
    };

    let properties: Vec<_> = fields
        .iter()
        .filter(|f| !f.serde_attrs.skip_deserializing() || !f.serde_attrs.skip_serializing())
        .map(|field| {
            if field.serde_attrs.flatten() {
                let (ty, type_def) = type_for_field_schema(field);

                let required = field.validation_attrs.required();

                let args = quote!(generator, #required);
                let mut schema_expr = quote_spanned! {ty.span()=>
                    schemars::_private::json_schema_for_flatten::<#ty>(#args)
                };

                prepend_type_def(type_def, &mut schema_expr);

                quote! {
                    schemars::_private::flatten(&mut schema, #schema_expr);
                }
            } else {
                let name = field.name();
                let default = field_default_expr(field, set_container_default.is_some());

                let (ty, type_def) = type_for_field_schema(field);

                let has_default = default.is_some();
                let required = field.validation_attrs.required();

                let metadata = SchemaMetadata {
                    read_only: field.serde_attrs.skip_deserializing(),
                    write_only: field.serde_attrs.skip_serializing(),
                    default,
                    ..field.attrs.as_metadata()
                };

                let generator = quote!(generator);
                let mut schema_expr = if field.validation_attrs.required() {
                    quote_spanned! {ty.span()=>
                        <#ty as schemars::JsonSchema>::_schemars_private_non_optional_json_schema(#generator)
                    }
                } else {
                    quote_spanned! {ty.span()=>
                        #generator.subschema_for::<#ty>()
                    }
                };

                metadata.apply_to_schema(&mut schema_expr);
                field.validation_attrs.apply_to_schema(&mut schema_expr);

                quote! {
                    {
                        #type_def
                        schemars::_private::insert_object_property::<#ty>(&mut schema, #name, #has_default, #required, #schema_expr);
                    }
                }}
            })
        .collect();

    let set_additional_properties = if deny_unknown_fields {
        quote! {
            "additionalProperties": false,
        }
    } else {
        TokenStream::new()
    };

    quote! ({
        #set_container_default
        let mut schema = schemars::json_schema!({
            "type": "object",
            #set_additional_properties
        });
        #(#properties)*
        schema
    })
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
                    fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
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

fn prepend_type_def(type_def: Option<TokenStream>, schema_expr: &mut TokenStream) {
    if let Some(type_def) = type_def {
        *schema_expr = quote! {
            {
                #type_def
                #schema_expr
            }
        }
    }
}
