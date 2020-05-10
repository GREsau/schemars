mod doc;
mod schemars_to_serde;

pub use doc::get_title_and_desc_from_doc;
pub use schemars_to_serde::process_serde_attrs;

use proc_macro2::{Group, Span, TokenStream, TokenTree};
use syn::parse::{self, Parse};

pub fn get_with_from_attrs(attrs: &[syn::Attribute]) -> Option<syn::Result<syn::Type>> {
    attrs
        .iter()
        .filter(|at| match at.path.get_ident() {
            // FIXME this is relying on order of attributes (schemars before serde) from schemars_to_serde.rs
            Some(i) => i == "schemars" || i == "serde",
            None => false,
        })
        .filter_map(get_with_from_attr)
        .next()
        .map(|lit| parse_lit_str(&lit))
}

fn get_with_from_attr(attr: &syn::Attribute) -> Option<syn::LitStr> {
    use syn::*;
    let nested_metas = match attr.parse_meta() {
        Ok(Meta::List(meta)) => meta.nested,
        _ => return None,
    };
    for nm in nested_metas {
        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
            path,
            lit: Lit::Str(with),
            ..
        })) = nm
        {
            if path.is_ident("with") {
                return Some(with);
            }
        }
    }
    None
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
