use super::{get_lit_str, get_meta_items, parse_lit_into_path, parse_lit_str};
use proc_macro2::TokenStream;
use serde_derive_internals::Ctxt;
use syn::{Expr, ExprLit, ExprPath, Lit, Meta, MetaNameValue, NestedMeta};

pub(crate) static VALIDATION_KEYWORDS: &[&str] = &[
    "range", "regex", "contains", "email", "phone", "url", "length", "required",
];

#[derive(Debug, Default)]
pub struct ValidationAttrs {
    pub length_min: Option<Expr>,
    pub length_max: Option<Expr>,
    pub length_equal: Option<Expr>,
    pub range_min: Option<Expr>,
    pub range_max: Option<Expr>,
    pub regex: Option<Expr>,
    pub contains: Option<String>,
    pub required: bool,
    pub format: Option<&'static str>,
}

impl ValidationAttrs {
    pub fn new(attrs: &[syn::Attribute], errors: &Ctxt) -> Self {
        // TODO allow setting "validate" attributes through #[schemars(...)]
        ValidationAttrs::default()
            .populate(attrs, "schemars", false, errors)
            .populate(attrs, "validate", true, errors)
    }

    fn populate(
        mut self,
        attrs: &[syn::Attribute],
        attr_type: &'static str,
        ignore_errors: bool,
        errors: &Ctxt,
    ) -> Self {
        let duplicate_error = |meta: &MetaNameValue| {
            if !ignore_errors {
                let msg = format!(
                    "duplicate schemars attribute `{}`",
                    meta.path.get_ident().unwrap()
                );
                errors.error_spanned_by(meta, msg)
            }
        };
        let mutual_exclusive_error = |meta: &MetaNameValue, other: &str| {
            if !ignore_errors {
                let msg = format!(
                    "schemars attribute cannot contain both `{}` and `{}`",
                    meta.path.get_ident().unwrap(),
                    other,
                );
                errors.error_spanned_by(meta, msg)
            }
        };

        for meta_item in attrs
            .iter()
            .flat_map(|attr| get_meta_items(attr, attr_type, errors))
            .flatten()
        {
            match &meta_item {
                NestedMeta::Meta(Meta::List(meta_list)) if meta_list.path.is_ident("length") => {
                    for nested in meta_list.nested.iter() {
                        match nested {
                            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("min") => {
                                if self.length_min.is_some() {
                                    duplicate_error(nv)
                                } else if self.length_equal.is_some() {
                                    mutual_exclusive_error(nv, "equal")
                                } else {
                                    self.length_min = str_or_num_to_expr(&errors, "min", &nv.lit);
                                }
                            }
                            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("max") => {
                                if self.length_max.is_some() {
                                    duplicate_error(nv)
                                } else if self.length_equal.is_some() {
                                    mutual_exclusive_error(nv, "equal")
                                } else {
                                    self.length_max = str_or_num_to_expr(&errors, "max", &nv.lit);
                                }
                            }
                            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("equal") => {
                                if self.length_equal.is_some() {
                                    duplicate_error(nv)
                                } else if self.length_min.is_some() {
                                    mutual_exclusive_error(nv, "min")
                                } else if self.length_max.is_some() {
                                    mutual_exclusive_error(nv, "max")
                                } else {
                                    self.length_equal =
                                        str_or_num_to_expr(&errors, "equal", &nv.lit);
                                }
                            }
                            meta => {
                                if !ignore_errors {
                                    errors.error_spanned_by(
                                        meta,
                                        format!("unknown item in schemars length attribute"),
                                    );
                                }
                            }
                        }
                    }
                }

                NestedMeta::Meta(Meta::List(meta_list)) if meta_list.path.is_ident("range") => {
                    for nested in meta_list.nested.iter() {
                        match nested {
                            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("min") => {
                                if self.range_min.is_some() {
                                    duplicate_error(nv)
                                } else {
                                    self.range_min = str_or_num_to_expr(&errors, "min", &nv.lit);
                                }
                            }
                            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("max") => {
                                if self.range_max.is_some() {
                                    duplicate_error(nv)
                                } else {
                                    self.range_max = str_or_num_to_expr(&errors, "max", &nv.lit);
                                }
                            }
                            meta => {
                                if !ignore_errors {
                                    errors.error_spanned_by(
                                        meta,
                                        format!("unknown item in schemars range attribute"),
                                    );
                                }
                            }
                        }
                    }
                }

                NestedMeta::Meta(Meta::Path(m))
                    if m.is_ident("required") || m.is_ident("required_nested") =>
                {
                    self.required = true;
                }

                // TODO cause compile error if format is already Some
                // FIXME #[validate(...)] overrides #[schemars(...)] - should be other way around!
                NestedMeta::Meta(Meta::Path(m)) if m.is_ident("email") => {
                    self.format = Some("email");
                }
                NestedMeta::Meta(Meta::Path(m)) if m.is_ident("url") => {
                    self.format = Some("uri");
                }
                NestedMeta::Meta(Meta::Path(m)) if m.is_ident("phone") => {
                    self.format = Some("phone");
                }

                // TODO cause compile error if regex/contains are specified more than once
                // FIXME #[validate(...)] overrides #[schemars(...)] - should be other way around!
                NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. }))
                    if path.is_ident("regex") =>
                {
                    self.regex = parse_lit_into_expr_path(errors, attr_type, "regex", lit).ok()
                }

                NestedMeta::Meta(Meta::List(meta_list)) if meta_list.path.is_ident("regex") => {
                    for x in meta_list.nested.iter() {
                        match x {
                            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                path, lit, ..
                            })) if path.is_ident("path") => {
                                self.regex =
                                    parse_lit_into_expr_path(errors, attr_type, "path", lit).ok()
                            }
                            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                path, lit, ..
                            })) if path.is_ident("pattern") => {
                                self.regex = get_lit_str(errors, attr_type, "pattern", lit)
                                    .ok()
                                    .map(|litstr| {
                                        Expr::Lit(syn::ExprLit {
                                            attrs: Vec::new(),
                                            lit: Lit::Str(litstr.clone()),
                                        })
                                    })
                            }
                            meta => {
                                if !ignore_errors {
                                    errors.error_spanned_by(
                                        meta,
                                        format!("unknown item in schemars regex attribute"),
                                    );
                                }
                            }
                        }
                    }
                }

                NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. }))
                    if path.is_ident("contains") =>
                {
                    self.contains = get_lit_str(errors, attr_type, "contains", lit)
                        .ok()
                        .map(|litstr| litstr.value())
                }

                NestedMeta::Meta(Meta::List(meta_list)) if meta_list.path.is_ident("contains") => {
                    for x in meta_list.nested.iter() {
                        match x {
                            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                path, lit, ..
                            })) if path.is_ident("pattern") => {
                                self.contains = get_lit_str(errors, attr_type, "contains", lit)
                                    .ok()
                                    .map(|litstr| litstr.value())
                            }
                            meta => {
                                if !ignore_errors {
                                    errors.error_spanned_by(
                                        meta,
                                        format!("unknown item in schemars contains attribute"),
                                    );
                                }
                            }
                        }
                    }
                }

                _ => {}
            }
        }
        self
    }

    pub fn apply_to_schema(&self, schema_expr: &mut TokenStream) {
        let mut array_validation = Vec::new();
        let mut number_validation = Vec::new();
        let mut object_validation = Vec::new();
        let mut string_validation = Vec::new();

        if let Some(length_min) = self
            .length_min
            .as_ref()
            .or_else(|| self.length_equal.as_ref())
        {
            string_validation.push(quote! {
                validation.min_length = Some(#length_min as u32);
            });
            array_validation.push(quote! {
                validation.min_items = Some(#length_min as u32);
            });
        }

        if let Some(length_max) = self
            .length_max
            .as_ref()
            .or_else(|| self.length_equal.as_ref())
        {
            string_validation.push(quote! {
                validation.max_length = Some(#length_max as u32);
            });
            array_validation.push(quote! {
                validation.max_items = Some(#length_max as u32);
            });
        }

        if let Some(range_min) = &self.range_min {
            number_validation.push(quote! {
                validation.minimum = Some(#range_min as f64);
            });
        }

        if let Some(range_max) = &self.range_max {
            number_validation.push(quote! {
                validation.maximum = Some(#range_max as f64);
            });
        }

        if let Some(regex) = &self.regex {
            string_validation.push(quote! {
                validation.pattern = Some(#regex.to_string());
            });
        }

        if let Some(contains) = &self.contains {
            object_validation.push(quote! {
                validation.required.insert(#contains.to_string());
            });

            if self.regex.is_none() {
                let pattern = crate::regex_syntax::escape(contains);
                string_validation.push(quote! {
                    validation.pattern = Some(#pattern.to_string());
                });
            }
        }

        let format = self.format.as_ref().map(|f| {
            quote! {
                schema_object.format = Some(#f.to_string());
            }
        });

        let array_validation = wrap_array_validation(array_validation);
        let number_validation = wrap_number_validation(number_validation);
        let object_validation = wrap_object_validation(object_validation);
        let string_validation = wrap_string_validation(string_validation);

        if array_validation.is_some()
            || number_validation.is_some()
            || object_validation.is_some()
            || string_validation.is_some()
            || format.is_some()
        {
            *schema_expr = quote! {
                {
                    let mut schema = #schema_expr;
                    if let schemars::schema::Schema::Object(schema_object) = &mut schema
                    {
                        #array_validation
                        #number_validation
                        #object_validation
                        #string_validation
                        #format
                    }
                    schema
                }
            }
        }
    }
}

fn parse_lit_into_expr_path(
    cx: &Ctxt,
    attr_type: &'static str,
    meta_item_name: &'static str,
    lit: &syn::Lit,
) -> Result<Expr, ()> {
    parse_lit_into_path(cx, attr_type, meta_item_name, lit).map(|path| {
        Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path,
        })
    })
}

fn wrap_array_validation(v: Vec<TokenStream>) -> Option<TokenStream> {
    if v.is_empty() {
        None
    } else {
        Some(quote! {
            if schema_object.has_type(schemars::schema::InstanceType::Array) {
                let validation = schema_object.array();
                #(#v)*
            }
        })
    }
}

fn wrap_number_validation(v: Vec<TokenStream>) -> Option<TokenStream> {
    if v.is_empty() {
        None
    } else {
        Some(quote! {
            if schema_object.has_type(schemars::schema::InstanceType::Integer)
                || schema_object.has_type(schemars::schema::InstanceType::Number) {
                let validation = schema_object.number();
                #(#v)*
            }
        })
    }
}

fn wrap_object_validation(v: Vec<TokenStream>) -> Option<TokenStream> {
    if v.is_empty() {
        None
    } else {
        Some(quote! {
            if schema_object.has_type(schemars::schema::InstanceType::Object) {
                let validation = schema_object.object();
                #(#v)*
            }
        })
    }
}

fn wrap_string_validation(v: Vec<TokenStream>) -> Option<TokenStream> {
    if v.is_empty() {
        None
    } else {
        Some(quote! {
            if schema_object.has_type(schemars::schema::InstanceType::String) {
                let validation = schema_object.string();
                #(#v)*
            }
        })
    }
}

fn str_or_num_to_expr(cx: &Ctxt, meta_item_name: &str, lit: &Lit) -> Option<Expr> {
    match lit {
        Lit::Str(s) => parse_lit_str::<ExprPath>(s).ok().map(Expr::Path),
        Lit::Int(_) | Lit::Float(_) => Some(Expr::Lit(ExprLit {
            attrs: Vec::new(),
            lit: lit.clone(),
        })),
        _ => {
            cx.error_spanned_by(
                lit,
                format!(
                    "expected `{}` to be a string or number literal",
                    meta_item_name
                ),
            );
            None
        }
    }
}
