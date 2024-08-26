use proc_macro2::{TokenStream, TokenTree};
use syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    Expr, ExprLit, Ident, Lit, LitStr, Meta, MetaNameValue,
};

use super::{path_str, AttrCtxt};

pub fn require_path_only(meta: Meta, cx: &AttrCtxt) -> Result<(), ()> {
    match meta {
        Meta::Path(_) => Ok(()),
        _ => {
            let name = path_str(meta.path());
            cx.error_spanned_by(
                meta,
                format_args!(
                    "unexpected value of {} {} attribute item",
                    cx.attr_type, name
                ),
            );
            Err(())
        }
    }
}

pub fn parse_name_value_expr(meta: Meta, cx: &AttrCtxt) -> Result<Expr, ()> {
    match meta {
        Meta::NameValue(m) => Ok(m.value),
        _ => {
            let name = path_str(meta.path());
            cx.error_spanned_by(
                meta,
                format_args!(
                    "expected {} {} attribute item to have a value: `{} = ...`",
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
                "expected {} {} attribute item to have a string value: `{} = \"...\"`",
                cx.attr_type, name, name
            ),
        );
        return Err(());
    };

    parse_lit_str(lit_str, cx)
}

fn parse_lit_str<T: Parse>(lit_str: LitStr, cx: &AttrCtxt) -> Result<T, ()> {
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
    let parser = Punctuated::<Extension, Token![,]>::parse_terminated;
    parse_meta_list(meta, cx, parser)
}

pub fn parse_length_or_range(outer_meta: Meta, cx: &AttrCtxt) -> Result<LengthOrRange, ()> {
    let outer_name = path_str(outer_meta.path());
    let mut result = LengthOrRange::default();

    for nested_meta in parse_nested_meta(outer_meta, cx)? {
        match nested_meta
            .path()
            .get_ident()
            .map(Ident::to_string)
            .unwrap_or_default()
            .as_str()
        {
            "min" => match (&result.min, &result.equal) {
                (Some(_), _) => cx.duplicate_error(&nested_meta),
                (_, Some(_)) => cx.mutual_exclusive_error(&nested_meta, "equal"),
                _ => result.min = parse_name_value_expr_handle_lit_str(nested_meta, cx).ok(),
            },
            "max" => match (&result.max, &result.equal) {
                (Some(_), _) => cx.duplicate_error(&nested_meta),
                (_, Some(_)) => cx.mutual_exclusive_error(&nested_meta, "equal"),
                _ => result.max = parse_name_value_expr_handle_lit_str(nested_meta, cx).ok(),
            },
            "equal" => match (&result.min, &result.max, &result.equal) {
                (Some(_), _, _) => cx.mutual_exclusive_error(&nested_meta, "min"),
                (_, Some(_), _) => cx.mutual_exclusive_error(&nested_meta, "max"),
                (_, _, Some(_)) => cx.duplicate_error(&nested_meta),
                _ => result.equal = parse_name_value_expr_handle_lit_str(nested_meta, cx).ok(),
            },
            unknown => {
                if cx.attr_type == "schemars" {
                    cx.error_spanned_by(
                        nested_meta,
                        format_args!(
                            "unknown item in schemars {} attribute: `{}`",
                            outer_name, unknown
                        ),
                    );
                }
            }
        }
    }

    Ok(result)
}

pub fn parse_regex(outer_meta: Meta, cx: &AttrCtxt) -> Result<Expr, ()> {
    todo!()
}

pub fn parse_nested_meta(meta: Meta, cx: &AttrCtxt) -> Result<impl IntoIterator<Item = Meta>, ()> {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    parse_meta_list(meta, cx, parser)
}

fn parse_meta_list<F: Parser>(meta: Meta, cx: &AttrCtxt, parser: F) -> Result<F::Output, ()> {
    let Meta::List(meta_list) = meta else {
        let name = path_str(meta.path());
        cx.error_spanned_by(
            meta,
            format_args!(
                "expected {} {} attribute item to be of the form `{}(...)`",
                cx.attr_type, name, name
            ),
        );
        return Err(());
    };

    meta_list.parse_args_with(parser).map_err(|err| {
        cx.syn_error(err);
    })
}

// Like `parse_name_value_expr`, but if the result is a string literal, then parse its contents.
pub fn parse_name_value_expr_handle_lit_str(meta: Meta, cx: &AttrCtxt) -> Result<Expr, ()> {
    let expr = parse_name_value_expr(meta, cx)?;

    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = expr
    {
        parse_lit_str(lit_str, cx)
    } else {
        Ok(expr)
    }
}

#[derive(Debug, Default)]
pub struct LengthOrRange {
    pub min: Option<Expr>,
    pub max: Option<Expr>,
    pub equal: Option<Expr>,
}

#[derive(Debug)]
pub struct Extension {
    pub key_str: String,
    pub key_lit: LitStr,
    pub value: TokenStream,
}

impl Parse for Extension {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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
