use proc_macro2::{TokenStream, TokenTree};
use syn::{
    parse::Parse, punctuated::Punctuated, Attribute, Expr, ExprLit, Lit, LitStr, Meta,
    MetaNameValue, Path, Type,
};

use super::{path_str, AttrCtxt};

pub fn name_value_expr(meta: Meta, cx: &AttrCtxt) -> Result<Expr, ()> {
    match meta {
        Meta::NameValue(m) => Ok(m.value),
        _ => {
            let name = path_str(meta.path());
            cx.error_spanned_by(
                meta,
                format_args!(
                    "expected {} {} attribute value to have a value: `{} = ...`",
                    cx.attr_type, name, name
                ),
            );
            Err(())
        }
    }
}

pub fn parse_name_value_lit_str<T: Parse>(meta: Meta, cx: &AttrCtxt) -> Result<T, ()> {
    let Meta::NameValue(MetaNameValue {
        value: Expr::Lit(ExprLit {
            lit: Lit::Str(lit_str),
            ..
        }),
        ..
    }) = meta
    else {
        let name = path_str(meta.path());
        cx.error_spanned_by(
            meta,
            format_args!(
                "expected {} {} attribute value to have a string value: `{} = \"...\"`",
                cx.attr_type, name, name
            ),
        );
        return Err(());
    };

    lit_str.parse().map_err(|_| {
        cx.error_spanned_by(
            &lit_str,
            format_args!(
                "failed to parse \"{}\" as a {}",
                lit_str.value(),
                std::any::type_name::<T>()
                    .rsplit("::")
                    .next()
                    .unwrap_or_default()
                    .to_ascii_lowercase(),
            ),
        );
    })
}

pub fn parse_extensions(
    meta: Meta,
    cx: &AttrCtxt,
) -> Result<impl IntoIterator<Item = Extension>, ()> {
    let Meta::List(meta) = meta else {
        let name = path_str(meta.path());
        cx.error_spanned_by(
            meta,
            format_args!(
                "expected {} {} attribute value to be of the form `{}(\"...\" = expr)`",
                cx.attr_type, name, name
            ),
        );
        return Err(());
    };

    let parser = Punctuated::<Extension, Token![,]>::parse_terminated;
    meta.parse_args_with(parser).map_err(|err| {
        cx.syn_error(err);
    })
}

#[derive(Debug)]
pub struct Extension {
    pub key_str: String,
    pub key_lit: LitStr,
    pub value: TokenStream,
}

impl Parse for Extension {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key = input.parse::<LitStr>()?;
        input.parse::<Token![=]>()?;
        let mut value = TokenStream::new();

        while !input.is_empty() && !input.peek(Token![,]) {
            value.extend([input.parse::<TokenTree>()?]);
        }

        if value.is_empty() {
            return Err(syn::Error::new(input.span(), "Expected extension value"));
        }

        Ok(Extension {
            key_str: key.value(),
            key_lit: key,
            value,
        })
    }
}
