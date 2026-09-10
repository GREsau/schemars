use proc_macro2::TokenStream;
use syn::Expr;

use crate::{idents::SCHEMA, schema_exprs::SchemaExpr};

use super::{
    parse_meta::{
        parse_contains, parse_length_or_range, parse_name_value_expr,
        parse_name_value_expr_handle_lit_str, parse_nested_meta, parse_pattern,
        parse_schemars_regex, parse_validate_regex, require_path_only, LengthOrRange,
    },
    AttrCtxt, CustomMeta,
};

#[derive(Clone, Copy, PartialEq)]
pub enum Format {
    Email,
    Uri,
    Ip,
    Ipv4,
    Ipv6,
}

impl Format {
    fn attr_str(self) -> &'static str {
        match self {
            Format::Email => "email",
            Format::Uri => "url",
            Format::Ip => "ip",
            Format::Ipv4 => "ipv4",
            Format::Ipv6 => "ipv6",
        }
    }

    fn schema_str(self) -> &'static str {
        match self {
            Format::Email => "email",
            Format::Uri => "uri",
            Format::Ip => "ip",
            Format::Ipv4 => "ipv4",
            Format::Ipv6 => "ipv6",
        }
    }

    fn from_attr_str(s: &str) -> Option<Self> {
        Some(match s {
            "email" => Format::Email,
            "url" => Format::Uri,
            "ip" => Format::Ip,
            "ipv4" => Format::Ipv4,
            "ipv6" => Format::Ipv6,
            _ => return None,
        })
    }
}

#[derive(Default)]
pub struct ValidationAttrs {
    // validator/garde style keywords
    pub length: Option<LengthOrRange>,
    pub range: Option<LengthOrRange>,
    pub pattern: Option<Expr>,
    pub regex: Option<Expr>,
    pub contains: Option<Expr>,
    pub required: bool,
    pub format: Option<Format>,
    pub inner: Option<Box<ValidationAttrs>>,
    // serde_valid style keywords
    pub minimum: Option<Expr>,
    pub maximum: Option<Expr>,
    pub exclusive_minimum: Option<Expr>,
    pub exclusive_maximum: Option<Expr>,
    pub multiple_of: Option<Expr>,
    pub min_length: Option<Expr>,
    pub max_length: Option<Expr>,
    pub min_items: Option<Expr>,
    pub max_items: Option<Expr>,
    pub unique_items: bool,
    pub min_properties: Option<Expr>,
    pub max_properties: Option<Expr>,
    pub enumeration: Option<Expr>,
}

impl ValidationAttrs {
    pub fn add_mutators(&self, expr: &mut SchemaExpr) {
        self.add_mutators2(&mut expr.mutators, &quote!(&mut #SCHEMA));
    }

    fn add_mutators2(&self, mutators: &mut Vec<TokenStream>, mut_ref_schema: &TokenStream) {
        if let Some(length) = &self.length {
            Self::add_length_or_range(length, mutators, "string", "Length", mut_ref_schema);
            Self::add_length_or_range(length, mutators, "array", "Items", mut_ref_schema);
        }

        if let Some(range) = &self.range {
            Self::add_length_or_range(range, mutators, "number", "imum", mut_ref_schema);
        }

        // `regex` (validator/schemars) only applies to string schemas.
        if let Some(regex) = &self.regex {
            mutators.push(quote! {
                schemars::_private::insert_validation_property(#mut_ref_schema, "string", "pattern", (#regex).to_string());
            });
        }

        // `pattern` includes serde_valid `pattern = ...`, which applies to each collection item.
        // On plain string schemas this behaves the same as `insert_validation_property`.
        if let Some(pattern) = &self.pattern {
            mutators.push(quote! {
                schemars::_private::insert_validation_property_or_items(#mut_ref_schema, "string", "pattern", (#pattern).to_string());
            });
        }

        if let Some(contains) = &self.contains {
            mutators.push(quote! {
                schemars::_private::must_contain(#mut_ref_schema, &#contains.to_string());
            });
        }

        if let Some(format) = &self.format {
            let f = format.schema_str();
            mutators.push(quote! {
                    (#mut_ref_schema).insert("format".into(), #f.into());
            });
        }

        if let Some(minimum) = &self.minimum {
            mutators.push(quote! {
                schemars::_private::insert_validation_property_or_items(#mut_ref_schema, "number", "minimum", #minimum);
            });
        }

        if let Some(maximum) = &self.maximum {
            mutators.push(quote! {
                schemars::_private::insert_validation_property_or_items(#mut_ref_schema, "number", "maximum", #maximum);
            });
        }

        if let Some(exclusive_minimum) = &self.exclusive_minimum {
            mutators.push(quote! {
                schemars::_private::insert_validation_property_or_items(#mut_ref_schema, "number", "exclusiveMinimum", #exclusive_minimum);
            });
        }

        if let Some(exclusive_maximum) = &self.exclusive_maximum {
            mutators.push(quote! {
                schemars::_private::insert_validation_property_or_items(#mut_ref_schema, "number", "exclusiveMaximum", #exclusive_maximum);
            });
        }

        if let Some(multiple_of) = &self.multiple_of {
            mutators.push(quote! {
                schemars::_private::insert_validation_property_or_items(#mut_ref_schema, "number", "multipleOf", #multiple_of);
            });
        }

        if let Some(min_length) = &self.min_length {
            mutators.push(quote! {
                schemars::_private::insert_validation_property_or_items(#mut_ref_schema, "string", "minLength", #min_length);
            });
        }

        if let Some(max_length) = &self.max_length {
            mutators.push(quote! {
                schemars::_private::insert_validation_property_or_items(#mut_ref_schema, "string", "maxLength", #max_length);
            });
        }

        if let Some(min_items) = &self.min_items {
            mutators.push(quote! {
                schemars::_private::insert_validation_property(#mut_ref_schema, "array", "minItems", #min_items);
            });
        }

        if let Some(max_items) = &self.max_items {
            mutators.push(quote! {
                schemars::_private::insert_validation_property(#mut_ref_schema, "array", "maxItems", #max_items);
            });
        }

        if let Some(min_properties) = &self.min_properties {
            mutators.push(quote! {
                schemars::_private::insert_validation_property(#mut_ref_schema, "object", "minProperties", #min_properties);
            });
        }

        if let Some(max_properties) = &self.max_properties {
            mutators.push(quote! {
                schemars::_private::insert_validation_property(#mut_ref_schema, "object", "maxProperties", #max_properties);
            });
        }

        if self.unique_items {
            mutators.push(quote! {
                schemars::_private::insert_validation_property(#mut_ref_schema, "array", "uniqueItems", true);
            });
        }

        if let Some(enumeration) = &self.enumeration {
            mutators.push(quote! {
                schemars::_private::insert_enum_validation(#mut_ref_schema, schemars::_private::serde_json::json!(#enumeration));
            });
        }

        if let Some(inner) = &self.inner {
            let mut inner_mutators = Vec::new();
            inner.add_mutators2(&mut inner_mutators, &quote!(inner_schema));

            if !inner_mutators.is_empty() {
                mutators.push(quote! {
                    schemars::_private::apply_inner_validation(#mut_ref_schema, |inner_schema| { #(#inner_mutators)* });
                });
            }
        }
    }

    fn add_length_or_range(
        value: &LengthOrRange,
        mutators: &mut Vec<TokenStream>,
        required_format: &str,
        key_suffix: &str,
        mut_ref_schema: &TokenStream,
    ) {
        if let Some(min) = value.min.as_ref().or(value.equal.as_ref()) {
            let key = format!("min{key_suffix}");
            mutators.push(quote!{
                schemars::_private::insert_validation_property(#mut_ref_schema, #required_format, #key, #min);
            });
        }

        if let Some(max) = value.max.as_ref().or(value.equal.as_ref()) {
            let key = format!("max{key_suffix}");
            mutators.push(quote!{
                schemars::_private::insert_validation_property(#mut_ref_schema, #required_format, #key, #max);
            });
        }
    }

    pub(super) fn populate(
        &mut self,
        schemars_cx: &mut AttrCtxt,
        validate_cx: &mut AttrCtxt,
        garde_cx: &mut AttrCtxt,
    ) {
        self.process_attr(schemars_cx);
        self.process_attr(validate_cx);
        self.process_attr(garde_cx);
    }

    fn process_attr(&mut self, cx: &mut AttrCtxt) {
        cx.parse_meta(|m, n, c| self.process_meta(m, n, c));
    }

    fn process_meta(
        &mut self,
        meta: CustomMeta,
        meta_name: &str,
        cx: &AttrCtxt,
    ) -> Result<(), CustomMeta> {
        if let Some(format) = Format::from_attr_str(meta_name) {
            self.handle_format(&meta, format, cx);
            return Ok(());
        }
        match meta_name {
            "length" => match self.length {
                Some(_) => cx.duplicate_error(&meta),
                None => self.length = parse_length_or_range(&meta, cx).ok(),
            },

            "range" => match self.range {
                Some(_) => cx.duplicate_error(&meta),
                None => self.range = parse_length_or_range(&meta, cx).ok(),
            },

            "required" => {
                if self.required {
                    cx.duplicate_error(&meta);
                } else if require_path_only(&meta, cx).is_ok() {
                    self.required = true;
                }
            }

            "pattern" => match (&self.pattern, &self.regex, &self.contains) {
                (Some(_p), _, _) => cx.duplicate_error(&meta),
                (_, Some(_r), _) => cx.mutual_exclusive_error(&meta, "regex"),
                (_, _, Some(_c)) => cx.mutual_exclusive_error(&meta, "contains"),
                (None, None, None) => match &meta {
                    // serde_valid-style: `#[validate(pattern = "...")]`
                    // schemars also allows the NameValue form
                    CustomMeta::NameValue(_) if cx.attr_type != "garde" => {
                        self.pattern = parse_name_value_expr(meta, cx).ok();
                    }
                    // garde/schemars-style: `pattern(...)`
                    CustomMeta::List(_) if cx.attr_type != "validate" => {
                        self.pattern = parse_pattern(&meta, cx).ok();
                    }
                    _ => return Err(meta),
                },
            },
            "regex" if cx.attr_type != "garde" => {
                match (&self.pattern, &self.regex, &self.contains) {
                    (Some(_p), _, _) => cx.mutual_exclusive_error(&meta, "pattern"),
                    (_, Some(_r), _) => cx.duplicate_error(&meta),
                    (_, _, Some(_c)) => cx.mutual_exclusive_error(&meta, "contains"),
                    (None, None, None) => {
                        if cx.attr_type == "validate" {
                            self.regex = parse_validate_regex(&meta, cx).ok();
                        } else {
                            self.regex = parse_schemars_regex(&meta, cx).ok();
                        }
                    }
                }
            }
            "contains" => match (&self.pattern, &self.regex, &self.contains) {
                (Some(_p), _, _) => cx.mutual_exclusive_error(&meta, "pattern"),
                (_, Some(_r), _) => cx.mutual_exclusive_error(&meta, "regex"),
                (_, _, Some(_c)) => cx.duplicate_error(&meta),
                (None, None, None) => self.contains = parse_contains(meta, cx).ok(),
            },

            "inner" if cx.attr_type != "validate" => {
                if let Ok(nested_meta) = parse_nested_meta(&meta, cx) {
                    let inner = self
                        .inner
                        .get_or_insert_with(|| Box::new(ValidationAttrs::default()));
                    let mut inner_cx = cx.new_nested_meta(nested_meta.into_iter().collect());
                    inner.process_attr(&mut inner_cx);
                }
            }

            // serde_valid / JSON Schema keywords (also allowed on `schemars(...)`)
            "minimum" => match self.minimum {
                Some(_) => cx.duplicate_error(&meta),
                None => self.minimum = parse_name_value_expr_handle_lit_str(meta, cx).ok(),
            },
            "maximum" => match self.maximum {
                Some(_) => cx.duplicate_error(&meta),
                None => self.maximum = parse_name_value_expr_handle_lit_str(meta, cx).ok(),
            },
            "exclusive_minimum" => match self.exclusive_minimum {
                Some(_) => cx.duplicate_error(&meta),
                None => {
                    self.exclusive_minimum = parse_name_value_expr_handle_lit_str(meta, cx).ok();
                }
            },
            "exclusive_maximum" => match self.exclusive_maximum {
                Some(_) => cx.duplicate_error(&meta),
                None => {
                    self.exclusive_maximum = parse_name_value_expr_handle_lit_str(meta, cx).ok();
                }
            },
            "multiple_of" => match self.multiple_of {
                Some(_) => cx.duplicate_error(&meta),
                None => self.multiple_of = parse_name_value_expr_handle_lit_str(meta, cx).ok(),
            },
            "min_length" => match self.min_length {
                Some(_) => cx.duplicate_error(&meta),
                None => self.min_length = parse_name_value_expr_handle_lit_str(meta, cx).ok(),
            },
            "max_length" => match self.max_length {
                Some(_) => cx.duplicate_error(&meta),
                None => self.max_length = parse_name_value_expr_handle_lit_str(meta, cx).ok(),
            },
            "min_items" => match self.min_items {
                Some(_) => cx.duplicate_error(&meta),
                None => self.min_items = parse_name_value_expr_handle_lit_str(meta, cx).ok(),
            },
            "max_items" => match self.max_items {
                Some(_) => cx.duplicate_error(&meta),
                None => self.max_items = parse_name_value_expr_handle_lit_str(meta, cx).ok(),
            },
            "min_properties" => match self.min_properties {
                Some(_) => cx.duplicate_error(&meta),
                None => self.min_properties = parse_name_value_expr_handle_lit_str(meta, cx).ok(),
            },
            "max_properties" => match self.max_properties {
                Some(_) => cx.duplicate_error(&meta),
                None => self.max_properties = parse_name_value_expr_handle_lit_str(meta, cx).ok(),
            },
            "enum" | "r#enum" => match self.enumeration {
                Some(_) => cx.duplicate_error(&meta),
                None => self.enumeration = parse_name_value_expr(meta, cx).ok(),
            },
            "unique_items" => {
                if self.unique_items {
                    cx.duplicate_error(&meta);
                } else if require_path_only(&meta, cx).is_ok() {
                    self.unique_items = true;
                }
            }

            // serde_valid-only items that do not affect the generated schema.
            // Recognised so that forms like `#[validate(minimum = 1, message = "...")]`
            // and `#[validate(custom = ...)]` are accepted without error.
            "message" | "message_fn" | "message_l10n" | "fluent" | "i18n" | "custom"
                if cx.attr_type == "validate" => {}

            _ => return Err(meta),
        }

        Ok(())
    }

    fn handle_format(&mut self, meta: &CustomMeta, format: Format, cx: &AttrCtxt) {
        match self.format {
            Some(current) if current == format => cx.duplicate_error(meta),
            Some(current) => cx.mutual_exclusive_error(meta, current.attr_str()),
            None => {
                // Allow a MetaList in validator attr (e.g. with message/code items),
                // but restrict it to path only in schemars attr.
                if cx.attr_type == "validate" || require_path_only(meta, cx).is_ok() {
                    self.format = Some(format);
                }
            }
        }
    }

    pub(crate) fn is_default(&self) -> bool {
        matches!(
            self,
            Self {
                contains: None,
                format: None,
                length: None,
                range: None,
                pattern: None,
                regex: None,
                required: false,
                inner: None,
                minimum: None,
                maximum: None,
                exclusive_minimum: None,
                exclusive_maximum: None,
                multiple_of: None,
                min_length: None,
                max_length: None,
                min_items: None,
                max_items: None,
                unique_items: false,
                min_properties: None,
                max_properties: None,
                enumeration: None,
            }
        )
    }
}
