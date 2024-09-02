use proc_macro2::{TokenStream, TokenTree};
use syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    Expr, ExprLit, Lit, LitStr, Meta, MetaList, MetaNameValue,
};

use super::{path_str, AttrCtxt};

pub fn require_path_only(meta: Meta, cx: &AttrCtxt) -> Result<(), ()> {
    match meta {
        Meta::Path(_) => Ok(()),
        Meta::List(MetaList {
            path, delimiter, ..
        }) => {
            let name = path_str(&path);
            cx.syn_error(syn::Error::new(
                delimiter.span().join(),
                format_args!(
                    "unexpected value of {} {} attribute item",
                    cx.attr_type, name
                ),
            ));
            Err(())
        }
        Meta::NameValue(MetaNameValue {
            path,
            eq_token,
            value,
        }) => {
            let name = path_str(&path);
            cx.error_spanned_by(
                quote!(#eq_token #value),
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
    parse_meta_list_with(&meta, cx, parser)
}

pub fn parse_length_or_range(outer_meta: Meta, cx: &AttrCtxt) -> Result<LengthOrRange, ()> {
    let outer_name = path_str(outer_meta.path());
    let mut result = LengthOrRange::default();

    for nested_meta in parse_nested_meta(outer_meta, cx)? {
        match path_str(nested_meta.path()).as_str() {
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
                            "unknown item in schemars {outer_name} attribute: `{unknown}`",
                        ),
                    );
                }
            }
        }
    }

    Ok(result)
}

pub fn parse_pattern(meta: Meta, cx: &AttrCtxt) -> Result<Expr, ()> {
    parse_meta_list_with(&meta, cx, Expr::parse)
}

pub fn parse_schemars_regex(outer_meta: Meta, cx: &AttrCtxt) -> Result<Expr, ()> {
    let mut pattern = None;

    for nested_meta in parse_nested_meta(outer_meta.clone(), cx)? {
        match path_str(nested_meta.path()).as_str() {
            "pattern" => match &pattern {
                Some(_) => cx.duplicate_error(&nested_meta),
                None => pattern = parse_name_value_expr(nested_meta, cx).ok(),
            },
            "path" => {
                cx.error_spanned_by(nested_meta, "`path` is not supported in `schemars(regex(...))` attribute - use `schemars(regex(pattern = ...))` instead")
            },
            unknown => {
                cx.error_spanned_by(
                    nested_meta,
                    format_args!("unknown item in schemars `regex` attribute: `{unknown}`"),
                );
            }
        }
    }

    pattern.ok_or_else(|| {
        cx.error_spanned_by(
            outer_meta,
            "`schemars(regex(...))` attribute requires `pattern = ...`",
        )
    })
}

pub fn parse_validate_regex(outer_meta: Meta, cx: &AttrCtxt) -> Result<Expr, ()> {
    let mut path = None;

    for nested_meta in parse_nested_meta(outer_meta.clone(), cx)? {
        match path_str(nested_meta.path()).as_str() {
            "path" => match &path{
                Some(_) => cx.duplicate_error(&nested_meta),
                None => path = parse_name_value_expr_handle_lit_str(nested_meta, cx).ok(),
            },
            "pattern" => {
                cx.error_spanned_by(nested_meta, "`pattern` is not supported in `validate(regex(...))` attribute - use either `validate(regex(path = ...))` or `schemars(regex(pattern = ...))` instead")
            },
            _ => {
                // ignore unknown properties in `validate` attribute
            }
        }
    }

    path.ok_or_else(|| {
        cx.error_spanned_by(
            outer_meta,
            "`validate(regex(...))` attribute requires `path = ...`",
        )
    })
}

pub fn parse_contains(outer_meta: Meta, cx: &AttrCtxt) -> Result<Expr, ()> {
    #[derive(Debug)]
    enum ContainsFormat {
        Metas(Punctuated<Meta, Token![,]>),
        Expr(Expr),
    }

    impl Parse for ContainsFormat {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            // An imperfect but good-enough heuristic for determining whether it looks more like a
            // comma-separated meta list (validator-style), or a single expression (garde-style).
            // This heuristic may not generalise well-enough for attributes other than `contains`!
            // `foo = bar` => Metas (not Expr::Assign)
            // `foo, bar`  => Metas
            // `foo`       => Expr (not Meta::Path)
            // `foo(bar)`  => Expr (not Meta::List)
            if input.peek2(Token![,]) || input.peek2(Token![=]) {
                Punctuated::parse_terminated(input).map(Self::Metas)
            } else {
                input.parse().map(Self::Expr)
            }
        }
    }

    let nested_meta_or_expr = match cx.attr_type {
        "validate" => parse_meta_list_with(&outer_meta, cx, Punctuated::parse_terminated)
            .map(ContainsFormat::Metas),
        "garde" => parse_meta_list_with(&outer_meta, cx, Expr::parse).map(ContainsFormat::Expr),
        "schemars" => parse_meta_list_with(&outer_meta, cx, ContainsFormat::parse),
        wat => {
            unreachable!("Unexpected attr type `{wat}` for `contains` item. This is a bug in schemars, please raise an issue!")
        }
    }?;

    let nested_metas = match nested_meta_or_expr {
        ContainsFormat::Expr(expr) => return Ok(expr),
        ContainsFormat::Metas(m) => m,
    };

    let mut pattern = None;

    for nested_meta in nested_metas {
        match path_str(nested_meta.path()).as_str() {
            "pattern" => match &pattern {
                Some(_) => cx.duplicate_error(&nested_meta),
                None => pattern = parse_name_value_expr(nested_meta, cx).ok(),
            },
            unknown => {
                if cx.attr_type == "schemars" {
                    cx.error_spanned_by(
                        nested_meta,
                        format_args!("unknown item in schemars `contains` attribute: `{unknown}`"),
                    );
                }
            }
        }
    }

    pattern.ok_or_else(|| {
        cx.error_spanned_by(
            outer_meta,
            "`contains` attribute item requires `pattern = ...`",
        )
    })
}

pub fn parse_nested_meta(meta: Meta, cx: &AttrCtxt) -> Result<impl IntoIterator<Item = Meta>, ()> {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    parse_meta_list_with(&meta, cx, parser)
}

fn parse_meta_list_with<F: Parser>(meta: &Meta, cx: &AttrCtxt, parser: F) -> Result<F::Output, ()> {
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
