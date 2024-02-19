use super::{get_lit_str, parse_lit_str, Attr, BoolAttr, Symbol};
use proc_macro2::TokenStream;
use quote::ToTokens;
use serde_derive_internals::Ctxt;
use syn::{meta::ParseNestedMeta, Expr, ExprLit, ExprPath, Lit};

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
    inner: Option<Box<Self>>,
}

const MIN: Symbol = "min";
const MAX: Symbol = "max";
const EQUAL: Symbol = "equal";
const REGEX: Symbol = "regex";
const CONTAINS: Symbol = "contains";
const REQUIRED: Symbol = "required";
const INNER: Symbol = "inner";
const FORMAT: Symbol = "format";
const PATH: Symbol = "path";
const PATTERN: Symbol = "pattern";

enum Regex {
    Pattern(syn::LitStr),
    Path(syn::Path),
}

struct Populate<'c> {
    length_min: Attr<'c, Expr>,
    length_max: Attr<'c, Expr>,
    length_equal: Attr<'c, Expr>,
    range_min: Attr<'c, Expr>,
    range_max: Attr<'c, Expr>,
    regex: Attr<'c, Regex>,
    contains: Attr<'c, String>,
    required: BoolAttr<'c>,
    format: Attr<'c, Format>,
    inner: Attr<'c, ValidationAttrs>,
    cx: &'c Ctxt,
}

impl<'c> From<Populate<'c>> for ValidationAttrs {
    fn from(pop: Populate<'c>) -> Self {
        Self {
            length_min: pop.length_min.get(),
            length_max: pop.length_max.get(),
            length_equal: pop.length_equal.get(),
            range_min: pop.range_min.get(),
            range_max: pop.range_max.get(),
            regex: pop.regex.get().map(|rx| match rx {
                Regex::Path(path) => Expr::Path(syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                }),
                Regex::Pattern(patt) => Expr::Lit(syn::ExprLit {
                    attrs: Vec::new(),
                    lit: syn::Lit::Str(patt),
                }),
            }),
            contains: pop.contains.get(),
            required: pop.required.get(),
            format: pop.format.get(),
            inner: pop.inner.get().map(Box::new),
        }
    }
}

impl<'c> Populate<'c> {
    fn new(cx: &'c Ctxt) -> Self {
        Self {
            length_min: Attr::none(cx, MIN),
            length_max: Attr::none(cx, MAX),
            length_equal: Attr::none(cx, EQUAL),
            range_min: Attr::none(cx, MIN),
            range_max: Attr::none(cx, MAX),
            regex: Attr::none(cx, REGEX),
            contains: Attr::none(cx, CONTAINS),
            required: BoolAttr::none(cx, REQUIRED),
            format: Attr::none(cx, FORMAT),
            inner: Attr::none(cx, INNER),
            cx,
        }
    }

    fn populate(&mut self, meta: ParseNestedMeta, ignore_errors: bool) -> syn::Result<()> {
        let path_str = super::ident_to_string!(meta);

        // Wrap in a closure so we _always_ consume the set of meta items, otherwise
        // we get cascading errors that obscure the root error
        let mut inner = |meta: &ParseNestedMeta| {
            match path_str.as_str() {
                // length(min = n, max = n, equal = n)
                "length" => meta.parse_nested_meta(|linner| {
                    if linner.path.is_ident(MIN) {
                        str_or_num_to_expr(
                            &mut self.length_min,
                            &linner,
                            [self.length_equal.excl()],
                            ignore_errors,
                        );
                    } else if linner.path.is_ident(MAX) {
                        str_or_num_to_expr(
                            &mut self.length_max,
                            &linner,
                            [self.length_equal.excl()],
                            ignore_errors,
                        );
                    } else if linner.path.is_ident(EQUAL) {
                        str_or_num_to_expr(
                            &mut self.length_equal,
                            &linner,
                            [self.length_min.excl(), self.length_max.excl()],
                            ignore_errors,
                        );
                    } else {
                        if !ignore_errors {
                            self.cx.error_spanned_by(
                                linner.path,
                                "unknown item in schemars length attribute",
                            );
                        }
                        super::skip_item(linner.input)?;
                    }

                    Ok(())
                })?,
                // range(min = n, max = n)
                "range" => {
                    meta.parse_nested_meta(|rinner| {
                        if rinner.path.is_ident(MIN) {
                            str_or_num_to_expr(&mut self.range_min, &rinner, None, ignore_errors);
                        } else if rinner.path.is_ident(MAX) {
                            str_or_num_to_expr(&mut self.range_max, &rinner, None, ignore_errors);
                        } else {
                            if !ignore_errors {
                                self.cx.error_spanned_by(
                                    rinner.path,
                                    "unknown item in schemars range attribute",
                                );
                            }
                            super::skip_item(rinner.input)?;
                        }

                        Ok(())
                    })?;
                }
                "required" | "required_nested" => {
                    self.required.set_true(meta.path.clone(), ignore_errors);
                }
                "email" => {
                    self.format
                        .set(meta.path.clone(), Format::Email, ignore_errors);
                }
                "url" => {
                    self.format
                        .set(meta.path.clone(), Format::Uri, ignore_errors);
                }
                "phone" => {
                    self.format
                        .set(meta.path.clone(), Format::Phone, ignore_errors);
                }
                REGEX => {
                    // (regex = "path")
                    //
                    // (regex(path = "path", pattern = "pattern")
                    let item = &mut self.regex;
                    let excl = [self.contains.excl()];
                    if meta.input.peek(Token![=]) {
                        if let Some(ls) = get_lit_str(self.cx, REGEX, meta)? {
                            item.set_exclusive(
                                meta.path.clone(),
                                Regex::Path(ls.parse::<syn::Path>()?),
                                excl,
                                ignore_errors,
                            );
                        }
                    } else {
                        meta.parse_nested_meta(|rinner| {
                            if rinner.path.is_ident(PATH) {
                                if let Some(ls) = get_lit_str(self.cx, PATH, &rinner)? {
                                    item.set_exclusive(
                                        rinner.path,
                                        Regex::Path(ls.parse::<syn::Path>()?),
                                        excl,
                                        ignore_errors,
                                    );
                                }
                            } else if rinner.path.is_ident(PATTERN) {
                                if let Some(patt) = get_lit_str(self.cx, PATTERN, &rinner)? {
                                    item.set_exclusive(
                                        rinner.path,
                                        Regex::Pattern(patt),
                                        excl,
                                        ignore_errors,
                                    );
                                }
                            } else {
                                if !ignore_errors {
                                    self.cx.error_spanned_by(
                                        rinner.path,
                                        "unknown item in schemars regex attribute",
                                    );
                                }
                                super::skip_item(rinner.input)?;
                            }

                            Ok(())
                        })?;
                    }
                }
                CONTAINS => {
                    // (contains = "pattern")
                    //
                    // (contains(pattern = "pattern")
                    let item = &mut self.contains;
                    let excl = [self.regex.excl()];

                    if meta.input.peek(Token![=]) {
                        if let Some(patt) = get_lit_str(self.cx, CONTAINS, meta)? {
                            item.set_exclusive(
                                meta.path.clone(),
                                patt.value(),
                                excl,
                                ignore_errors,
                            );
                        }
                    } else {
                        meta.parse_nested_meta(|rinner| {
                            if rinner.path.is_ident(PATTERN) {
                                if let Some(patt) = get_lit_str(self.cx, PATTERN, &rinner)? {
                                    item.set_exclusive(
                                        rinner.path,
                                        patt.value(),
                                        excl,
                                        ignore_errors,
                                    );
                                }
                            } else {
                                if !ignore_errors {
                                    self.cx.error_spanned_by(
                                        rinner.path,
                                        "unknown item in schemars contains attribute",
                                    );
                                }

                                super::skip_item(rinner.input)?;
                            }

                            Ok(())
                        })?;
                    }
                }
                "inner" => {
                    let mut inner_pop = Populate::new(self.cx);

                    meta.parse_nested_meta(|iinner| inner_pop.populate(iinner, ignore_errors))?;

                    self.inner
                        .set(meta.path.clone(), inner_pop.into(), ignore_errors);
                }
                _ => {}
            }

            Ok(())
        };

        let res = inner(&meta);

        // We've already validated the contents of the serde/schemars attributes at this point
        // so we can just silently skip the items that don't affect validation
        super::skip_item(meta.input)?;

        res
    }
}

impl ValidationAttrs {
    pub fn new(attrs: &[syn::Attribute], cx: &Ctxt) -> Self {
        if attrs.is_empty() {
            return ValidationAttrs::default();
        }

        let mut pop = Populate::new(cx);

        // Note that we can't just iterate through the attributes once, to preserve
        // the old logic we need to walk through the schemars ones first, then
        // validate, as the schemars attributes will give errors and validate ones
        // won't, and won't override an attribute if it was already set by schemars
        for (name, ignore_errors) in [("schemars", false), ("validate", true)] {
            for attr in attrs {
                if !attr.path().is_ident(name) {
                    continue;
                }

                match &attr.meta {
                    syn::Meta::Path(p) => {
                        // Due to how the old code gathered meta items, this would work, but parse_nested_meta
                        // requires a list or name value pair
                        if !p.is_ident("validate") {
                            cx.error_spanned_by(p, "unexpected path item");
                        }

                        continue;
                    }
                    syn::Meta::List(ml) => {
                        if ml.tokens.is_empty() {
                            continue;
                        }
                    }
                    syn::Meta::NameValue(mnv) => {
                        cx.error_spanned_by(mnv, "only meta lists are supported");
                        continue;
                    }
                }

                if let Err(err) = attr.parse_nested_meta(|meta| pop.populate(meta, ignore_errors)) {
                    cx.syn_error(err);
                }
            }
        }

        pop.into()
    }

    pub fn required(&self) -> bool {
        self.required
    }

    pub fn apply_to_schema(&self, schema_expr: &mut TokenStream) {
        if let Some(apply_expr) = self.apply_to_schema_expr() {
            *schema_expr = quote! {
                {
                    let mut schema = #schema_expr;
                    #apply_expr
                    schema
                }
            }
        }
    }

    fn apply_to_schema_expr(&self) -> Option<TokenStream> {
        let mut array_validation = Vec::new();
        let mut number_validation = Vec::new();
        let mut object_validation = Vec::new();
        let mut string_validation = Vec::new();

        if let Some(length_min) = self.length_min.as_ref().or(self.length_equal.as_ref()) {
            string_validation.push(quote! {
                validation.min_length = Some(#length_min as u32);
            });
            array_validation.push(quote! {
                validation.min_items = Some(#length_min as u32);
            });
        }

        if let Some(length_max) = self.length_max.as_ref().or(self.length_equal.as_ref()) {
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
            let f = f.schema_str();
            quote! {
                schema_object.format = Some(#f.to_string());
            }
        });

        let inner_validation = self
            .inner
            .as_deref()
            .and_then(|inner| inner.apply_to_schema_expr())
            .map(|apply_expr| {
                quote! {
                    if schema_object.has_type(schemars::schema::InstanceType::Array) {
                        if let Some(schemars::schema::SingleOrVec::Single(inner_schema)) = &mut schema_object.array().items {
                            let mut schema = &mut **inner_schema;
                            #apply_expr
                        }
                    }
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
            || inner_validation.is_some()
        {
            Some(quote! {
                if let schemars::schema::Schema::Object(schema_object) = &mut schema {
                    #array_validation
                    #number_validation
                    #object_validation
                    #string_validation
                    #format
                    #inner_validation
                }
            })
        } else {
            None
        }
    }
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

fn str_or_num_to_expr(
    attr: &mut Attr<'_, Expr>,
    meta: &syn::meta::ParseNestedMeta<'_>,
    excl: impl IntoIterator<Item = (Symbol, bool)>,
    ie: bool,
) {
    let val = match meta.value() {
        Ok(lit) => lit,
        Err(err) => {
            attr.cx.syn_error(err);
            return;
        }
    };

    let lit = match val.parse() {
        Ok(l) => l,
        Err(err) => {
            attr.cx.syn_error(err);
            return;
        }
    };

    let expr = match lit {
        Lit::Str(s) => {
            let Some(expr) = parse_lit_str::<ExprPath>(&s).ok().map(Expr::Path) else {
                return;
            };
            expr
        }
        Lit::Int(_) | Lit::Float(_) => Expr::Lit(ExprLit {
            attrs: Vec::new(),
            lit,
        }),
        _ => {
            attr.cx.error_spanned_by(
                lit,
                format!(
                    "expected `{}` to be a string or number literal",
                    meta.path.to_token_stream().to_string().replace(' ', "")
                ),
            );
            return;
        }
    };

    attr.set_exclusive(meta.path.clone(), expr, excl, ie);
}
