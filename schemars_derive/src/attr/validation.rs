use proc_macro2::TokenStream;
use syn::{Expr, Meta};

use crate::idents::SCHEMA;

use super::{
    parse_meta::{
        parse_contains, parse_length_or_range, parse_nested_meta, parse_schemars_regex,
        parse_validate_regex, require_path_only, LengthOrRange,
    },
    AttrCtxt,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Email,
    Uri,
}

impl Format {
    fn attr_str(self) -> &'static str {
        match self {
            Format::Email => "email",
            Format::Uri => "url",
        }
    }

    fn schema_str(self) -> &'static str {
        match self {
            Format::Email => "email",
            Format::Uri => "uri",
        }
    }

    fn from_attr_str(s: &str) -> Self {
        match s {
            "email" => Format::Email,
            "url" => Format::Uri,
            _ => {
                panic!("Invalid format attr string `{s}`. This is a bug in schemars, please raise an issue!")
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ValidationAttrs {
    pub length: Option<LengthOrRange>,
    pub range: Option<LengthOrRange>,
    pub regex: Option<Expr>,
    pub contains: Option<Expr>,
    pub required: bool,
    pub format: Option<Format>,
    pub inner: Option<Box<ValidationAttrs>>,
}

impl ValidationAttrs {
    pub fn add_mutators(&self, mutators: &mut Vec<TokenStream>) {
        self.add_mutators2(mutators, &quote!(&mut #SCHEMA));
    }

    fn add_mutators2(&self, mutators: &mut Vec<TokenStream>, mut_ref_schema: &TokenStream) {
        if let Some(length) = &self.length {
            Self::add_length_or_range(length, mutators, "string", "Length", mut_ref_schema);
            Self::add_length_or_range(length, mutators, "array", "Items", mut_ref_schema);
        }

        if let Some(range) = &self.range {
            Self::add_length_or_range(range, mutators, "number", "imum", mut_ref_schema);
        }

        if let Some(regex) = &self.regex {
            mutators.push(quote! {
                schemars::_private::insert_validation_property(#mut_ref_schema, "string", "pattern", (#regex).to_string());
            });
        }

        if let Some(contains) = &self.contains {
            mutators.push(quote! {
                schemars::_private::must_contain(#mut_ref_schema, #contains.to_string());
            });
        }

        if let Some(format) = &self.format {
            let f = format.schema_str();
            mutators.push(quote! {
                    (#mut_ref_schema).ensure_object().insert("format".into(), #f.into());
            })
        };

        if let Some(inner) = &self.inner {
            let mut inner_mutators = Vec::new();
            inner.add_mutators2(&mut inner_mutators, &quote!(inner_schema));

            if !inner_mutators.is_empty() {
                mutators.push(quote! {
                    schemars::_private::apply_inner_validation(#mut_ref_schema, |inner_schema| { #(#inner_mutators)* });
                })
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

    pub(super) fn populate(&mut self, schemars_cx: &mut AttrCtxt, validate_cx: &mut AttrCtxt) {
        self.process_attr(schemars_cx);
        self.process_attr(validate_cx);
    }

    fn process_attr(&mut self, cx: &mut AttrCtxt) {
        cx.parse_meta(|m, n, c| self.process_meta(m, n, c));
    }

    fn process_meta(&mut self, meta: Meta, meta_name: &str, cx: &AttrCtxt) -> Option<Meta> {
        match meta_name {
            "length" => match self.length {
                Some(_) => cx.duplicate_error(&meta),
                None => self.length = parse_length_or_range(meta, cx).ok(),
            },

            "range" => match self.range {
                Some(_) => cx.duplicate_error(&meta),
                None => self.range = parse_length_or_range(meta, cx).ok(),
            },

            "email" | "url" => self.handle_format(meta, meta_name, cx),

            "required" => {
                if self.required {
                    cx.duplicate_error(&meta);
                } else if require_path_only(meta, cx).is_ok() {
                    self.required = true;
                }
            }

            "regex" => match (&self.regex, &self.contains, cx.attr_type) {
                (Some(_), _, _) => cx.duplicate_error(&meta),
                (_, Some(_), _) => cx.mutual_exclusive_error(&meta, "contains"),
                (None, None, "schemars") => self.regex = parse_schemars_regex(meta, cx).ok(),
                (None, None, "validate") => self.regex = parse_validate_regex(meta, cx).ok(),
                (None, None, wat) => {
                    panic!("Unexpected attr type `{wat}` for regex item. This is a bug in schemars, please raise an issue!")
                }
            },
            "contains" => match (&self.regex, &self.contains) {
                (Some(_), _) => cx.mutual_exclusive_error(&meta, "regex"),
                (_, Some(_)) => cx.duplicate_error(&meta),
                (None, None) => self.contains = parse_contains(meta, cx).ok(),
            },

            "inner" => {
                if let Ok(nested_meta) = parse_nested_meta(meta, cx) {
                    let inner = self
                        .inner
                        .get_or_insert_with(|| Box::new(ValidationAttrs::default()));
                    let mut inner_cx = cx.new_nested_meta(nested_meta.into_iter().collect());
                    inner.process_attr(&mut inner_cx);
                }
            }

            _ => return Some(meta),
        }

        None
    }

    fn handle_format(&mut self, meta: Meta, meta_name: &str, cx: &AttrCtxt) {
        match &self.format {
            Some(f) if f.attr_str() == meta_name => cx.duplicate_error(&meta),
            Some(f) => cx.mutual_exclusive_error(&meta, f.attr_str()),
            None => {
                // FIXME this is too strict - it may be a MetaList in validator attr (e.g. with message/code items)
                if require_path_only(meta, cx).is_ok() {
                    self.format = Some(Format::from_attr_str(meta_name))
                }
            }
        }
    }
}
