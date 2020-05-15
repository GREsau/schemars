mod doc;
mod schemars_to_serde;

pub use doc::get_title_and_desc_from_doc;
pub use schemars_to_serde::process_serde_attrs;

use proc_macro2::{Group, Span, TokenStream, TokenTree};
use serde_derive_internals::Ctxt;
use syn::parse::{self, Parse};
use syn::Meta::{List, NameValue};
use syn::MetaNameValue;
use syn::NestedMeta::{Lit, Meta};

#[derive(Debug, Default)]
pub struct Attrs {
    pub with: Option<WithAttr>,
    pub title: Option<String>,
    pub description: Option<String>,
    // TODO pub example: Option<syn::Path>,
}

#[derive(Debug)]
pub enum WithAttr {
    Type(syn::Type),
    Function(syn::Path),
}

impl Attrs {
    pub fn new(attrs: &[syn::Attribute], errors: &Ctxt) -> Self {
        let (title, description) = doc::get_title_and_desc_from_doc(attrs);
        Attrs {
            title,
            description,
            ..Attrs::default()
        }
        .populate(attrs, "schemars", false, errors)
        .populate(attrs, "serde", true, errors)
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
                Meta(NameValue(m)) if m.path.is_ident("with") => {
                    if let Ok(ty) = parse_lit_into_ty(errors, attr_type, "with", &m.lit) {
                        match self.with {
                            Some(WithAttr::Type(_)) => duplicate_error(m),
                            Some(WithAttr::Function(_)) => mutual_exclusive_error(m, "schema_with"),
                            None => self.with = Some(WithAttr::Type(ty)),
                        }
                    }
                }

                Meta(NameValue(m)) if m.path.is_ident("schema_with") => {
                    if let Ok(fun) = parse_lit_into_path(errors, attr_type, "schema_with", &m.lit) {
                        match self.with {
                            Some(WithAttr::Function(_)) => duplicate_error(m),
                            Some(WithAttr::Type(_)) => mutual_exclusive_error(m, "with"),
                            None => self.with = Some(WithAttr::Function(fun)),
                        }
                    }
                }

                Meta(_meta_item) => {
                    // TODO uncomment this for 0.8.0 (breaking change)
                    //  https://github.com/GREsau/schemars/issues/18
                    // if !ignore_errors {
                    //     let path = meta_item
                    //         .path()
                    //         .into_token_stream()
                    //         .to_string()
                    //         .replace(' ', "");
                    //     errors.error_spanned_by(
                    //         meta_item.path(),
                    //         format!("unknown schemars container attribute `{}`", path),
                    //     );
                    // }
                }

                Lit(_lit) => {
                    // TODO uncomment this for 0.8.0 (breaking change)
                    //  https://github.com/GREsau/schemars/issues/18
                    // if !ignore_errors {
                    //     errors.error_spanned_by(
                    //         lit,
                    //         "unexpected literal in schemars container attribute",
                    //     );
                    // }
                }
            }
        }
        self
    }
}

fn get_meta_items(
    attr: &syn::Attribute,
    attr_type: &'static str,
    errors: &Ctxt,
) -> Result<Vec<syn::NestedMeta>, ()> {
    if !attr.path.is_ident(attr_type) {
        return Ok(Vec::new());
    }

    match attr.parse_meta() {
        Ok(List(meta)) => Ok(meta.nested.into_iter().collect()),
        Ok(other) => {
            errors.error_spanned_by(other, format!("expected #[{}(...)]", attr_type));
            Err(())
        }
        Err(err) => {
            errors.error_spanned_by(attr, err);
            Err(())
        }
    }
}

fn get_lit_str<'a>(
    cx: &Ctxt,
    attr_type: &'static str,
    meta_item_name: &'static str,
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(lit) = lit {
        Ok(lit)
    } else {
        cx.error_spanned_by(
            lit,
            format!(
                "expected {} {} attribute to be a string: `{} = \"...\"`",
                attr_type, meta_item_name, meta_item_name
            ),
        );
        Err(())
    }
}

fn parse_lit_into_ty(
    cx: &Ctxt,
    attr_type: &'static str,
    meta_item_name: &'static str,
    lit: &syn::Lit,
) -> Result<syn::Type, ()> {
    let string = get_lit_str(cx, attr_type, meta_item_name, lit)?;

    parse_lit_str(string).map_err(|_| {
        cx.error_spanned_by(
            lit,
            format!(
                "failed to parse type: `{} = {:?}`",
                meta_item_name,
                string.value()
            ),
        )
    })
}

fn parse_lit_into_path(
    cx: &Ctxt,
    attr_type: &'static str,
    meta_item_name: &'static str,
    lit: &syn::Lit,
) -> Result<syn::Path, ()> {
    let string = get_lit_str(cx, attr_type, meta_item_name, lit)?;

    parse_lit_str(string).map_err(|_| {
        cx.error_spanned_by(
            lit,
            format!(
                "failed to parse path: `{} = {:?}`",
                meta_item_name,
                string.value()
            ),
        )
    })
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
