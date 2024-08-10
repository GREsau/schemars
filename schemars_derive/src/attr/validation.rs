use super::{expr_as_lit_str, get_meta_items, parse_lit_into_path, parse_lit_str};
use proc_macro2::TokenStream;
use quote::ToTokens;
use serde_derive_internals::Ctxt;
use syn::{
    parse::Parser, punctuated::Punctuated, Expr, ExprPath, Lit, Meta, MetaList, MetaNameValue, Path,
};

pub(crate) static VALIDATION_KEYWORDS: &[&str] = &[
    "range", "regex", "contains", "email", "phone", "url", "length", "required",
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum Format {
    Email,
    Uri,
    Phone,
}

impl Format {
    fn attr_str(self) -> &'static str {
        match self {
            Format::Email => "email",
            Format::Uri => "url",
            Format::Phone => "phone",
        }
    }

    fn schema_str(self) -> &'static str {
        match self {
            Format::Email => "email",
            Format::Uri => "uri",
            Format::Phone => "phone",
        }
    }
}

#[derive(Debug, Default)]
pub struct ValidationAttrs {
    length_min: Option<Expr>,
    length_max: Option<Expr>,
    length_equal: Option<Expr>,
    range_min: Option<Expr>,
    range_max: Option<Expr>,
    regex: Option<Expr>,
    contains: Option<String>,
    required: bool,
    format: Option<Format>,
    inner: Option<Box<ValidationAttrs>>,
}

impl ValidationAttrs {
    pub fn new(attrs: &[syn::Attribute], errors: &Ctxt) -> Self {
        let schemars_items = get_meta_items(attrs, "schemars", errors, false);
        let validate_items = get_meta_items(attrs, "validate", errors, true);

        ValidationAttrs::default()
            .populate(schemars_items, "schemars", false, errors)
            .populate(validate_items, "validate", true, errors)
    }

    pub fn required(&self) -> bool {
        self.required
    }

    fn populate(
        mut self,
        meta_items: Vec<Meta>,
        attr_type: &'static str,
        ignore_errors: bool,
        errors: &Ctxt,
    ) -> Self {
        let duplicate_error = |path: &Path| {
            if !ignore_errors {
                let msg = format!(
                    "duplicate schemars attribute `{}`",
                    path.get_ident().unwrap()
                );
                errors.error_spanned_by(path, msg)
            }
        };
        let mutual_exclusive_error = |path: &Path, other: &str| {
            if !ignore_errors {
                let msg = format!(
                    "schemars attribute cannot contain both `{}` and `{}`",
                    path.get_ident().unwrap(),
                    other,
                );
                errors.error_spanned_by(path, msg)
            }
        };
        let duplicate_format_error = |existing: Format, new: Format, path: &syn::Path| {
            if !ignore_errors {
                let msg = if existing == new {
                    format!("duplicate schemars attribute `{}`", existing.attr_str())
                } else {
                    format!(
                        "schemars attribute cannot contain both `{}` and `{}`",
                        existing.attr_str(),
                        new.attr_str(),
                    )
                };
                errors.error_spanned_by(path, msg)
            }
        };
        let parse_nested_meta = |meta_list: MetaList| {
            let parser = Punctuated::<syn::Meta, Token![,]>::parse_terminated;
            match parser.parse2(meta_list.tokens) {
                Ok(p) => p,
                Err(e) => {
                    if !ignore_errors {
                        errors.syn_error(e);
                    }
                    Default::default()
                }
            }
        };

        for meta_item in meta_items {
            match meta_item {
                Meta::List(meta_list) if meta_list.path.is_ident("length") => {
                    for nested in parse_nested_meta(meta_list) {
                        match nested {
                            Meta::NameValue(nv) if nv.path.is_ident("min") => {
                                if self.length_min.is_some() {
                                    duplicate_error(&nv.path)
                                } else if self.length_equal.is_some() {
                                    mutual_exclusive_error(&nv.path, "equal")
                                } else {
                                    self.length_min = str_or_num_to_expr(errors, "min", nv.value);
                                }
                            }
                            Meta::NameValue(nv) if nv.path.is_ident("max") => {
                                if self.length_max.is_some() {
                                    duplicate_error(&nv.path)
                                } else if self.length_equal.is_some() {
                                    mutual_exclusive_error(&nv.path, "equal")
                                } else {
                                    self.length_max = str_or_num_to_expr(errors, "max", nv.value);
                                }
                            }
                            Meta::NameValue(nv) if nv.path.is_ident("equal") => {
                                if self.length_equal.is_some() {
                                    duplicate_error(&nv.path)
                                } else if self.length_min.is_some() {
                                    mutual_exclusive_error(&nv.path, "min")
                                } else if self.length_max.is_some() {
                                    mutual_exclusive_error(&nv.path, "max")
                                } else {
                                    self.length_equal =
                                        str_or_num_to_expr(errors, "equal", nv.value);
                                }
                            }
                            meta => {
                                if !ignore_errors {
                                    errors.error_spanned_by(
                                        meta,
                                        "unknown item in schemars length attribute".to_string(),
                                    );
                                }
                            }
                        }
                    }
                }

                Meta::List(meta_list) if meta_list.path.is_ident("range") => {
                    for nested in parse_nested_meta(meta_list) {
                        match nested {
                            Meta::NameValue(nv) if nv.path.is_ident("min") => {
                                if self.range_min.is_some() {
                                    duplicate_error(&nv.path)
                                } else {
                                    self.range_min = str_or_num_to_expr(errors, "min", nv.value);
                                }
                            }
                            Meta::NameValue(nv) if nv.path.is_ident("max") => {
                                if self.range_max.is_some() {
                                    duplicate_error(&nv.path)
                                } else {
                                    self.range_max = str_or_num_to_expr(errors, "max", nv.value);
                                }
                            }
                            meta => {
                                if !ignore_errors {
                                    errors.error_spanned_by(
                                        meta,
                                        "unknown item in schemars range attribute".to_string(),
                                    );
                                }
                            }
                        }
                    }
                }

                Meta::Path(m) if m.is_ident("required") || m.is_ident("required_nested") => {
                    self.required = true;
                }

                Meta::Path(p) if p.is_ident(Format::Email.attr_str()) => match self.format {
                    Some(f) => duplicate_format_error(f, Format::Email, &p),
                    None => self.format = Some(Format::Email),
                },
                Meta::Path(p) if p.is_ident(Format::Uri.attr_str()) => match self.format {
                    Some(f) => duplicate_format_error(f, Format::Uri, &p),
                    None => self.format = Some(Format::Uri),
                },
                Meta::Path(p) if p.is_ident(Format::Phone.attr_str()) => match self.format {
                    Some(f) => duplicate_format_error(f, Format::Phone, &p),
                    None => self.format = Some(Format::Phone),
                },

                Meta::NameValue(nv) if nv.path.is_ident("regex") => {
                    match (&self.regex, &self.contains) {
                        (Some(_), _) => duplicate_error(&nv.path),
                        (None, Some(_)) => mutual_exclusive_error(&nv.path, "contains"),
                        (None, None) => {
                            self.regex =
                                parse_lit_into_expr_path(errors, attr_type, "regex", &nv.value).ok()
                        }
                    }
                }

                Meta::List(meta_list) if meta_list.path.is_ident("regex") => {
                    match (&self.regex, &self.contains) {
                        (Some(_), _) => duplicate_error(&meta_list.path),
                        (None, Some(_)) => mutual_exclusive_error(&meta_list.path, "contains"),
                        (None, None) => {
                            for x in parse_nested_meta(meta_list) {
                                match x {
                                    Meta::NameValue(MetaNameValue { path, value, .. })
                                        if path.is_ident("path") =>
                                    {
                                        self.regex = parse_lit_into_expr_path(
                                            errors, attr_type, "path", &value,
                                        )
                                        .ok()
                                    }
                                    Meta::NameValue(MetaNameValue { path, value, .. })
                                        if path.is_ident("pattern") =>
                                    {
                                        self.regex =
                                            expr_as_lit_str(errors, attr_type, "pattern", &value)
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
                                                "unknown item in schemars regex attribute"
                                                    .to_string(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("contains") => {
                    match (&self.contains, &self.regex) {
                        (Some(_), _) => duplicate_error(&path),
                        (None, Some(_)) => mutual_exclusive_error(&path, "regex"),
                        (None, None) => {
                            self.contains = expr_as_lit_str(errors, attr_type, "contains", &value)
                                .map(|litstr| litstr.value())
                                .ok()
                        }
                    }
                }

                Meta::List(meta_list) if meta_list.path.is_ident("contains") => {
                    match (&self.contains, &self.regex) {
                        (Some(_), _) => duplicate_error(&meta_list.path),
                        (None, Some(_)) => mutual_exclusive_error(&meta_list.path, "regex"),
                        (None, None) => {
                            for x in parse_nested_meta(meta_list) {
                                match x {
                                    Meta::NameValue(MetaNameValue { path, value, .. })
                                        if path.is_ident("pattern") =>
                                    {
                                        self.contains =
                                            expr_as_lit_str(errors, attr_type, "contains", &value)
                                                .ok()
                                                .map(|litstr| litstr.value())
                                    }
                                    meta => {
                                        if !ignore_errors {
                                            errors.error_spanned_by(
                                                meta,
                                                "unknown item in schemars contains attribute"
                                                    .to_string(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Meta::List(meta_list) if meta_list.path.is_ident("inner") => match self.inner {
                    Some(_) => duplicate_error(&meta_list.path),
                    None => {
                        let inner_attrs = ValidationAttrs::default().populate(
                            parse_nested_meta(meta_list).into_iter().collect(),
                            attr_type,
                            ignore_errors,
                            errors,
                        );
                        self.inner = Some(Box::new(inner_attrs));
                    }
                },

                _ => {}
            }
        }
        self
    }

    pub fn apply_to_schema(&self, schema_expr: &mut TokenStream) {
        let setters = self.make_setters(quote!(&mut schema));
        if !setters.is_empty() {
            *schema_expr = quote!({
                let mut schema = #schema_expr;
                #(#setters)*
                schema
            });
        }
    }

    fn make_setters(&self, mut_schema: impl ToTokens) -> Vec<TokenStream> {
        let mut result = Vec::new();

        if let Some(length_min) = self.length_min.as_ref().or(self.length_equal.as_ref()) {
            result.push(quote! {
                schemars::_private::insert_validation_property(#mut_schema, "string", "minLength", #length_min);
            });
            result.push(quote! {
                schemars::_private::insert_validation_property(#mut_schema, "array", "minItems", #length_min);
            });
        }

        if let Some(length_max) = self.length_max.as_ref().or(self.length_equal.as_ref()) {
            result.push(quote! {
                schemars::_private::insert_validation_property(#mut_schema, "string", "maxLength", #length_max);
            });
            result.push(quote! {
                schemars::_private::insert_validation_property(#mut_schema, "array", "maxItems", #length_max);
            });
        }

        if let Some(range_min) = &self.range_min {
            result.push(quote! {
                schemars::_private::insert_validation_property(#mut_schema, "number", "minimum", #range_min);
            });
        }

        if let Some(range_max) = &self.range_max {
            result.push(quote! {
                schemars::_private::insert_validation_property(#mut_schema, "number", "maximum", #range_max);
            });
        }

        if let Some(regex) = &self.regex {
            result.push(quote! {
                schemars::_private::insert_validation_property(#mut_schema, "string", "pattern", #regex);
            });
        }

        if let Some(contains) = &self.contains {
            result.push(quote! {
                schemars::_private::append_required(#mut_schema, #contains);
            });

            if self.regex.is_none() {
                let pattern = crate::regex_syntax::escape(contains);
                result.push(quote! {
                    schemars::_private::insert_validation_property(#mut_schema, "string", "pattern", #pattern);
                });
            }
        }

        if let Some(format) = &self.format {
            let f = format.schema_str();
            result.push(quote! {
                schema.ensure_object().insert("format".to_owned(), #f.into());
            })
        };

        if let Some(inner) = &self.inner {
            let inner_setters = inner.make_setters(quote!(schema));
            if !inner_setters.is_empty() {
                result.push(quote! {
                    schemars::_private::apply_inner_validation(#mut_schema, |schema| { #(#inner_setters)* });
                })
            }
        }

        result
    }
}

fn parse_lit_into_expr_path(
    cx: &Ctxt,
    attr_type: &'static str,
    meta_item_name: &'static str,
    lit: &Expr,
) -> Result<Expr, ()> {
    parse_lit_into_path(cx, attr_type, meta_item_name, lit).map(|path| {
        Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path,
        })
    })
}

fn str_or_num_to_expr(cx: &Ctxt, meta_item_name: &str, expr: Expr) -> Option<Expr> {
    // this odd double-parsing is to make `-10` parsed as an Lit instead of an Expr::Unary
    let lit: Lit = match syn::parse2(expr.to_token_stream()) {
        Ok(l) => l,
        Err(err) => {
            cx.syn_error(err);
            return None;
        }
    };

    match lit {
        Lit::Str(s) => parse_lit_str::<ExprPath>(&s).ok().map(Expr::Path),
        Lit::Int(_) | Lit::Float(_) => Some(expr),
        _ => {
            cx.error_spanned_by(
                &expr,
                format!(
                    "expected `{}` to be a string or number literal, not {:?}",
                    meta_item_name, &expr
                ),
            );
            None
        }
    }
}
