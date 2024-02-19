macro_rules! ident_to_string {
    ($meta:expr) => {{
        let Some(path_str) = $meta.path.get_ident().map(|i| i.to_string()) else {
            return Err(syn::Error::new_spanned($meta.path, "expected valid ident"));
        };

        path_str
    }};
}

pub(crate) use ident_to_string;

mod doc;
mod schemars_to_serde;
mod validation;

pub use schemars_to_serde::process_serde_attrs;
pub use validation::ValidationAttrs;

use crate::metadata::SchemaMetadata;
use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::ToTokens;
use serde_derive_internals::Ctxt;
use syn::meta::ParseNestedMeta;
use syn::parse::{self, Parse};

// FIXME using the same struct for containers+variants+fields means that
//  with/schema_with are accepted (but ignored) on containers, and
//  repr/crate_name are accepted (but ignored) on variants and fields etc.

#[derive(Debug, Default)]
pub struct Attrs {
    pub with: Option<WithAttr>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub deprecated: bool,
    pub examples: Vec<syn::Path>,
    pub repr: Option<syn::Type>,
    pub crate_name: Option<syn::Path>,
    pub is_renamed: bool,
}

#[derive(Debug)]
pub enum WithAttr {
    Type(syn::Type),
    Function(syn::Path),
}

impl Attrs {
    pub fn new(attrs: &[syn::Attribute], cx: &Ctxt) -> Self {
        let mut result = Self::populate(attrs, &[("schemars", false), ("serde", true)], cx);

        result.deprecated = attrs.iter().any(|a| a.path().is_ident("deprecated"));
        result.repr = attrs
            .iter()
            .find(|a| a.path().is_ident("repr"))
            .and_then(|a| a.parse_args().ok());

        let (doc_title, doc_description) = doc::get_title_and_desc_from_doc(attrs);
        result.title = result.title.or(doc_title);
        result.description = result.description.or(doc_description);

        result
    }

    pub fn as_metadata(&self) -> SchemaMetadata<'_> {
        #[allow(clippy::ptr_arg)]
        fn none_if_empty(s: &String) -> Option<&str> {
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }

        SchemaMetadata {
            title: self.title.as_ref().and_then(none_if_empty),
            description: self.description.as_ref().and_then(none_if_empty),
            deprecated: self.deprecated,
            examples: &self.examples,
            read_only: false,
            write_only: false,
            default: None,
        }
    }

    fn populate(attrs: &[syn::Attribute], pop: &[(&'static str, bool)], cx: &Ctxt) -> Self {
        if attrs.is_empty() {
            return Self::default();
        }

        const WITH: Symbol = "with";
        const SCHEMA_WITH: Symbol = "schema_with";
        const TITLE: Symbol = "title";
        const DESCRIPTION: Symbol = "description";
        const RENAME: Symbol = "rename";
        const CRATE: Symbol = "crate";
        const EXAMPLE: Symbol = "example";

        let mut with = Attr::none(cx, WITH);
        let mut schema_with = Attr::none(cx, SCHEMA_WITH);
        let mut title = Attr::none(cx, TITLE);
        let mut description = Attr::none(cx, DESCRIPTION);
        let mut rename = BoolAttr::none(cx, RENAME);
        let mut krate = Attr::none(cx, CRATE);
        let mut examples = Vec::new();

        for attr in attrs {
            let Some((attr_type, ignore_errors)) = pop
                .iter()
                .find_map(|(n, i)| attr.path().is_ident(n).then_some((*n, *i)))
            else {
                continue;
            };

            let is_schema_rs = attr_type == "schemars";

            let syn::Meta::List(ml) = &attr.meta else {
                cx.error_spanned_by(
                    attr.path(),
                    format!("expected {attr_type} to be a attribute meta list"),
                );
                continue;
            };

            if ml.tokens.is_empty() || is_schema_rs && ml.path.is_ident("inner") {
                continue;
            }

            let mut boop = None;
            let res = attr.parse_nested_meta(|meta| {
                let path_str = ident_to_string!(meta);
                let mut skip = false;

                boop = Some(path_str.clone());

                match path_str.as_str() {
                    WITH => {
                        if let Some(ls) = get_lit_str(cx, WITH, &meta)? {
                            with.set_exclusive(
                                meta.path,
                                ls.parse()?,
                                [schema_with.excl()],
                                ignore_errors,
                            );
                        }
                    }
                    SCHEMA_WITH => {
                        if let Some(ls) = get_lit_str(cx, SCHEMA_WITH, &meta)? {
                            schema_with.set_exclusive(
                                meta.path,
                                ls.parse()?,
                                [with.excl()],
                                ignore_errors,
                            );
                        }
                    }
                    TITLE => {
                        if let Some(ls) = get_lit_str(cx, TITLE, &meta)? {
                            title.set(meta.path, ls.value(), ignore_errors);
                        }
                    }
                    DESCRIPTION => {
                        if let Some(ls) = get_lit_str(cx, DESCRIPTION, &meta)? {
                            description.set(meta.path, ls.value(), ignore_errors);
                        }
                    }
                    "example" => {
                        if let Some(ls) = get_lit_str(cx, EXAMPLE, &meta)? {
                            examples.push(ls.parse()?);
                        }
                    }
                    "rename" => {
                        rename.set_true(meta.path, ignore_errors);
                        skip = true;
                    }
                    "crate" if is_schema_rs => {
                        if let Some(ls) = get_lit_str(cx, DESCRIPTION, &meta)? {
                            krate.set(meta.path, ls.parse()?, ignore_errors);
                        }
                    }
                    "inner" => {
                        meta.parse_nested_meta(|inn| {
                            if inn.input.peek(syn::token::Paren) {
                                inn.parse_nested_meta(|inn2| skip_item(inn2.input))
                            } else {
                                skip_item(inn.input)
                            }
                        })?;
                    }
                    _ => {
                        if !schemars_to_serde::SERDE_KEYWORDS
                            .iter()
                            .chain(validation::VALIDATION_KEYWORDS)
                            .any(|kw| meta.path.is_ident(&kw))
                        {
                            let path = meta.path.to_token_stream().to_string().replace(' ', "");
                            cx.error_spanned_by(
                                meta.path,
                                format!("unknown schemars attribute `{path}`"),
                            );
                            return Ok(());
                        }

                        if meta.input.peek(syn::token::Paren) {
                            meta.parse_nested_meta(|inn| skip_item(inn.input))?;
                        }

                        skip = true;
                    }
                }

                if skip {
                    skip_item(meta.input)?;
                }

                Ok(())
            });

            if let Err(res) = res {
                cx.syn_error(res);
            }
        }

        Self {
            with: with
                .get()
                .map(WithAttr::Type)
                .or_else(|| schema_with.get().map(WithAttr::Function)),
            title: title.get(),
            description: description.get(),
            deprecated: false,
            repr: None,
            examples,
            crate_name: krate.get(),
            is_renamed: rename.get(),
        }
    }

    pub fn is_default(&self) -> bool {
        matches!(self, Self {
                with: None,
                title: None,
                description: None,
                deprecated: false,
                examples,
                repr: None,
                crate_name: None,
                is_renamed: _,
            } if examples.is_empty())
    }
}

type Symbol = &'static str;

struct Attr<'c, T> {
    cx: &'c Ctxt,
    name: Symbol,
    tokens: TokenStream,
    value: Option<T>,
}

impl<'c, T> Attr<'c, T> {
    fn none(cx: &'c Ctxt, name: Symbol) -> Self {
        Self {
            cx,
            name,
            tokens: TokenStream::new(),
            value: None,
        }
    }

    fn set_exclusive<A: ToTokens>(
        &mut self,
        obj: A,
        value: T,
        exl: impl IntoIterator<Item = (&'static str, bool)>,
        ie: bool,
    ) {
        let tokens = obj.into_token_stream();

        if self.value.is_some() {
            if !ie {
                self.cx.error_spanned_by(
                    tokens.clone(),
                    format!("duplicate schemars attribute `{}`", self.name),
                );
            }

            return;
        }

        let non_exclusive = exl.into_iter().fold(false, |acc, (name, set)| {
            if set && !ie {
                self.cx.error_spanned_by(
                    tokens.clone(),
                    format!(
                        "schemars attribute cannot contain both `{}` and `{name}`",
                        self.name
                    ),
                );
            }

            acc || set
        });

        if non_exclusive {
            return;
        }

        // The old implemenation just overwrites
        self.tokens = tokens;
        self.value = Some(value);
    }

    fn set<A: ToTokens>(&mut self, obj: A, value: T, ie: bool) {
        self.set_exclusive(obj, value, [], ie)
    }

    fn get(self) -> Option<T> {
        self.value
    }

    fn excl(&self) -> (&'static str, bool) {
        (self.name, self.value.is_some())
    }
}

struct BoolAttr<'c>(Attr<'c, ()>);

impl<'c> BoolAttr<'c> {
    fn none(cx: &'c Ctxt, name: Symbol) -> Self {
        Self(Attr::none(cx, name))
    }

    fn set_true<A: ToTokens>(&mut self, obj: A, ie: bool) {
        self.0.set(obj, (), ie);
    }

    fn get(&self) -> bool {
        self.0.value.is_some()
    }
}

fn skip_item(input: syn::parse::ParseStream) -> syn::Result<()> {
    // Advance past this meta item
    if input.peek(syn::Token![=]) {
        input.parse::<syn::Token![=]>()?;
        input.parse::<syn::Expr>()?;
    } else if input.peek(syn::token::Paren) {
        let _skip;
        syn::parenthesized!(_skip in input);
    }

    Ok(())
}

fn get_lit_str(
    cx: &Ctxt,
    attr_name: &'static str,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::LitStr>> {
    get_lit_str2(cx, attr_name, attr_name, meta)
}

fn get_lit_str2(
    cx: &Ctxt,
    attr_name: &'static str,
    meta_item_name: &'static str,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::LitStr>> {
    let expr: syn::Expr = meta.value()?.parse()?;
    let mut value = &expr;
    while let syn::Expr::Group(e) = value {
        value = &e.expr;
    }
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit),
        ..
    }) = value
    {
        let suffix = lit.suffix();
        if !suffix.is_empty() {
            cx.error_spanned_by(
                lit,
                format!("unexpected suffix `{}` on string literal", suffix),
            );
        }
        Ok(Some(lit.clone()))
    } else {
        cx.error_spanned_by(
            expr,
            format!(
                "expected serde {} attribute to be a string: `{} = \"...\"`",
                attr_name, meta_item_name
            ),
        );
        Ok(None)
    }
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
