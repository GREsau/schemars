// mod schemars_to_serde;
mod doc;
mod parse_meta;
mod validation;

// pub use schemars_to_serde::process_serde_attrs;
// pub use validation::ValidationAttrs;

use parse_meta::{name_value_expr, parse_extensions, parse_name_value_lit_str};
use proc_macro2::TokenStream;
use quote::ToTokens;
use serde_derive_internals::Ctxt;
use syn::Ident;
use syn::{punctuated::Punctuated, Attribute, Expr, ExprLit, Lit, Meta, Path, Type};
use validation::ValidationAttrs;

#[derive(Debug, Default)]
pub struct CommonAttrs {
    pub doc: Option<Expr>,
    pub deprecated: bool,
    pub title: Option<Expr>,
    pub description: Option<Expr>,
    pub examples: Vec<Path>,
    pub extensions: Vec<(String, TokenStream)>,
    pub transforms: Vec<Expr>,
}

#[derive(Debug, Default)]
pub struct FieldAttrs {
    pub common: CommonAttrs,
    pub with: Option<WithAttr>,
    pub validation: ValidationAttrs,
}

#[derive(Debug, Default)]
pub struct ContainerAttrs {
    pub common: CommonAttrs,
    pub repr: Option<Type>,
    pub crate_name: Option<Path>,
}

#[derive(Debug, Default)]
pub struct VariantAttrs {
    pub common: CommonAttrs,
    pub with: Option<WithAttr>,
}

#[derive(Debug)]
pub enum WithAttr {
    Type(Type),
    Function(Path),
}

impl CommonAttrs {
    fn populate(&mut self, attrs: &[Attribute], schemars_cx: &mut AttrCtxt) {
        self.populate_from_schemars(schemars_cx);

        self.doc = doc::get_doc(attrs);
        self.deprecated = attrs.iter().any(|a| a.path().is_ident("deprecated"));
    }

    fn populate_from_schemars(&mut self, cx: &mut AttrCtxt) {
        cx.parse_meta(|meta, meta_name, cx| {
            match meta_name
            {
                "title" => match self.title {
                    Some(_) => cx.duplicate_error(&meta),
                    None => self.title = name_value_expr(meta, cx).ok(),
                },

                "description" => match self.description {
                    Some(_) => cx.duplicate_error(&meta),
                    None => self.description = name_value_expr(meta, cx).ok(),
                },

                "example" => {
                    self.examples.extend(parse_name_value_lit_str(meta, cx));
                }

                "extend" => {
                    for ex in parse_extensions(meta, cx).into_iter().flatten() {
                        // This is O(n^2) but should be fine with the typically small number of extensions.
                        // If this does become a problem, it can be changed to use IndexMap, or a separate Map with cloned keys.
                        if self.extensions.iter().any(|e| e.0 == ex.key_str) {
                            cx.error_spanned_by(
                                ex.key_lit,
                                format_args!("Duplicate extension key '{}'", ex.key_str),
                            );
                        } else {
                            self.extensions.push((ex.key_str, ex.value));
                        }
                    }
                }

                "transform" => {
                    if let Ok(expr) = name_value_expr(meta, cx) {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }) = &expr
                        {
                            if lit_str.parse::<Expr>().is_ok() {
                                cx.error_spanned_by(
                                    &expr,
                                    format_args!(
                                        "Expected a `fn(&mut Schema)` or other value implementing `schemars::transform::Transform`, found `&str`.\nDid you mean `[schemars(transform = {})]`?",
                                        lit_str.value()
                                    ),
                                )
                            }
                        } else {
                            self.transforms.push(expr);
                        }
                    }
                }

                _ => return Some(meta),
            }

            None
        });
    }

    fn is_default(&self) -> bool {
        matches!(
            self,
            Self {
                title: None,
                description: None,
                doc: None,
                deprecated: false,
                examples,
                extensions,
                transforms,
            } if examples.is_empty() && extensions.is_empty() && transforms.is_empty()
        )
    }
}

impl FieldAttrs {
    pub fn new(attrs: &[Attribute], cx: &Ctxt) -> Self {
        let mut result = Self::default();
        result.populate(attrs, cx);
        result
    }

    fn populate(&mut self, attrs: &[Attribute], cx: &Ctxt) {
        let mut schemars_cx = AttrCtxt::new(cx, attrs, "schemars");

        self.common.populate(attrs, &mut schemars_cx);
        self.populate_from_schemars_or_serde(&mut schemars_cx);
        self.populate_from_schemars_or_serde(&mut AttrCtxt::new(cx, attrs, "serde"));

        // TODO validation
    }

    fn populate_from_schemars_or_serde(&mut self, cx: &mut AttrCtxt) {
        cx.parse_meta(|meta, meta_name, cx| {
            match meta_name {
                "with" => match self.with {
                    Some(WithAttr::Type(_)) => cx.duplicate_error(&meta),
                    Some(WithAttr::Function(_)) => cx.mutual_exclusive_error(&meta, "schema_with"),
                    None => self.with = parse_name_value_lit_str(meta, cx).ok().map(WithAttr::Type),
                },
                "schema_with" if cx.attr_type == "schemars" => match self.with {
                    Some(WithAttr::Function(_)) => cx.duplicate_error(&meta),
                    Some(WithAttr::Type(_)) => cx.mutual_exclusive_error(&meta, "with"),
                    None => {
                        self.with = parse_name_value_lit_str(meta, cx)
                            .ok()
                            .map(WithAttr::Function)
                    }
                },

                _ => return Some(meta),
            }

            None
        });
    }
}

impl ContainerAttrs {
    pub fn new(attrs: &[Attribute], cx: &Ctxt) -> Self {
        let mut result = Self::default();
        result.populate(attrs, cx);
        result
    }

    fn populate(&mut self, attrs: &[Attribute], cx: &Ctxt) {
        let mut schemars_cx = AttrCtxt::new(cx, attrs, "schemars");

        self.common.populate(attrs, &mut schemars_cx);
        self.populate_from_schemars(&mut schemars_cx);

        self.repr = attrs
            .iter()
            .find(|a| a.path().is_ident("repr"))
            .and_then(|a| a.parse_args().ok());
    }

    fn populate_from_schemars(&mut self, cx: &mut AttrCtxt) {
        cx.parse_meta(|meta, meta_name, cx| {
            match meta_name {
                "crate" => match self.crate_name {
                    Some(_) => cx.duplicate_error(&meta),
                    None => self.crate_name = parse_name_value_lit_str(meta, cx).ok(),
                },

                _ => return Some(meta),
            };

            None
        });
    }
}

impl VariantAttrs {
    pub fn new(attrs: &[Attribute], cx: &Ctxt) -> Self {
        let mut result = Self::default();
        result.populate(attrs, cx);
        result
    }

    fn populate(&mut self, attrs: &[Attribute], cx: &Ctxt) {
        let mut schemars_cx = AttrCtxt::new(cx, attrs, "schemars");

        self.common.populate(attrs, &mut schemars_cx);
        self.populate_from_schemars_or_serde(&mut schemars_cx);
        self.populate_from_schemars_or_serde(&mut AttrCtxt::new(cx, attrs, "serde"));
    }

    fn populate_from_schemars_or_serde(&mut self, cx: &mut AttrCtxt) {
        cx.parse_meta(|meta, meta_name, cx| {
            match meta_name {
                "with" => match self.with {
                    Some(WithAttr::Type(_)) => cx.duplicate_error(&meta),
                    Some(WithAttr::Function(_)) => cx.mutual_exclusive_error(&meta, "schema_with"),
                    None => self.with = parse_name_value_lit_str(meta, cx).ok().map(WithAttr::Type),
                },
                "schema_with" if cx.attr_type == "schemars" => match self.with {
                    Some(WithAttr::Function(_)) => cx.duplicate_error(&meta),
                    Some(WithAttr::Type(_)) => cx.mutual_exclusive_error(&meta, "with"),
                    None => {
                        self.with = parse_name_value_lit_str(meta, cx)
                            .ok()
                            .map(WithAttr::Function)
                    }
                },

                _ => return Some(meta),
            }

            None
        });
    }

    pub fn is_default(&self) -> bool {
        matches!(
            self,
            Self {
                common,
                with: None,
            } if common.is_default()
        )
    }
}

fn get_meta_items(attrs: &[Attribute], attr_type: &'static str, cx: &Ctxt) -> Vec<Meta> {
    let mut result = vec![];

    for attr in attrs.iter().filter(|a| a.path().is_ident(attr_type)) {
        match attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
            Ok(list) => result.extend(list),
            Err(err) => {
                if attr_type == "schemars" {
                    cx.syn_error(err)
                }
            }
        }
    }

    result
}

fn path_str(path: &Path) -> String {
    path.get_ident()
        .map(Ident::to_string)
        .unwrap_or_else(|| path.into_token_stream().to_string().replace(' ', ""))
}

struct AttrCtxt<'a> {
    inner: &'a Ctxt,
    attr_type: &'static str,
    metas: Vec<Meta>,
}

impl<'a> AttrCtxt<'a> {
    pub fn new(inner: &'a Ctxt, attrs: &'a [Attribute], attr_type: &'static str) -> Self {
        Self {
            inner,
            attr_type,
            metas: get_meta_items(attrs, attr_type, inner),
        }
    }

    pub fn parse_meta(&mut self, mut handle: impl FnMut(Meta, &str, &Self) -> Option<Meta>) {
        let metas = std::mem::take(&mut self.metas);
        self.metas = metas
            .into_iter()
            .filter_map(|meta| {
                meta.path()
                    .get_ident()
                    .map(Ident::to_string)
                    .and_then(|name| handle(meta, &name, self))
            })
            .collect();
    }

    pub fn error_spanned_by<A: ToTokens, T: std::fmt::Display>(&self, obj: A, msg: T) {
        self.inner.error_spanned_by(obj, msg);
    }

    pub fn syn_error(&self, err: syn::Error) {
        self.inner.syn_error(err);
    }

    pub fn mutual_exclusive_error(&self, meta: &Meta, other_attr: &str) {
        if self.attr_type == "schemars" {
            self.error_spanned_by(
                meta,
                format_args!(
                    "schemars attribute cannot contain both `{}` and `{}`",
                    path_str(meta.path()),
                    other_attr,
                ),
            );
        }
    }

    pub fn duplicate_error(&self, meta: &Meta) {
        if self.attr_type == "schemars" {
            self.error_spanned_by(
                &meta,
                format_args!("duplicate schemars attribute `{}`", path_str(meta.path())),
            );
        }
    }
}

impl Drop for AttrCtxt<'_> {
    fn drop(&mut self) {
        if self.attr_type == "schemars" {
            for unhandled_meta in &self.metas {
                self.error_spanned_by(
                    unhandled_meta.path(),
                    format_args!(
                        "unknown schemars attribute `{}`",
                        path_str(unhandled_meta.path())
                    ),
                );
            }
        }
    }
}
