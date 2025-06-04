use proc_macro2::TokenStream;
use serde_derive_internals::Ctxt;
use std::collections::BTreeSet;
use syn::Ident;

use crate::ast::Container;

pub fn get_rename_format_type_params(
    errors: &Ctxt,
    meta: &syn::Meta,
    format_string: &str,
) -> BTreeSet<Ident> {
    if !format_string.contains('{') {
        return BTreeSet::new();
    }

    let name = format_string.replace("{{", "").replace("}}", "");

    let mut result = BTreeSet::new();

    let mut segments = name.split('{');

    if segments.next().unwrap_or_default().contains('}') {
        // The name format string contains a '}' before the first '{'
        errors.error_spanned_by(meta, "invalid name format string: unmatched `}` found");
    }

    for segment in segments {
        match segment.split_once('}') {
            Some((param, rest)) => {
                if rest.contains('}') {
                    errors
                        .error_spanned_by(meta, "invalid name format string: unmatched `}` found");
                }

                match syn::parse_str(param) {
                    Ok(id) => {
                        result.insert(id);
                    }
                    Err(_) => errors.error_spanned_by(
                        meta,
                        format_args!(
                            "invalid name format string: expected generic param, found `{param}`"
                        ),
                    ),
                }
            }
            None => {
                errors.error_spanned_by(
                    meta,
                    "invalid name format string: found `{` without matching `}`",
                );
            }
        }
    }

    result
}

pub fn get_schema_name_and_id_exprs(
    cont: &Container,
    relevant_type_params: BTreeSet<Ident>,
) -> (TokenStream, TokenStream) {
    let const_params = Vec::from_iter(cont.generics.const_params().map(|c| &c.ident));

    let is_renamed = cont.attrs.raw_rename.is_some();
    let mut simple_name = cont.serde_attrs.name().deserialize_name().to_string();

    if !is_renamed {
        if let Some(path) = cont.serde_attrs.remote() {
            if let Some(segment) = path.segments.last() {
                simple_name = segment.ident.to_string();
            }
        }
    }

    if is_renamed && !cont.rename_params.is_empty() {
        let (const_params, type_params): (Vec<_>, Vec<_>) = cont
            .rename_params
            .iter()
            .partition(|i| const_params.contains(i));

        (
            quote! {
                schemars::_private::alloc::borrow::Cow::Owned(
                    schemars::_private::alloc::format!(
                        #simple_name
                        #(,#type_params=#type_params::schema_name())*
                        #(,#const_params=#const_params)*)
                )
            },
            quote! {
                schemars::_private::alloc::borrow::Cow::Owned(
                    schemars::_private::alloc::format!(
                        ::core::concat!(
                            ::core::module_path!(),
                            "::",
                            #simple_name
                        )
                        #(,#type_params=#type_params::schema_id())*
                        #(,#const_params=#const_params)*
                    )
                )
            },
        )
    } else if is_renamed || (relevant_type_params.is_empty() && const_params.is_empty()) {
        (
            quote! {
                schemars::_private::alloc::borrow::Cow::Borrowed(#simple_name)
            },
            quote! {
                schemars::_private::alloc::borrow::Cow::Borrowed(::core::concat!(
                    ::core::module_path!(),
                    "::",
                    #simple_name
                ))
            },
        )
    } else {
        let schema_name_fmt = format!(
            "{}_for_{}",
            simple_name,
            &"_and_{}".repeat(relevant_type_params.len() + const_params.len())[5..]
        );

        // FIXME instead of `relevant_type_params`, this should use field types (from relevant_field_type_predicates, but without the `for <'...>` shenanigans)
        let schema_id_fmt = format!(
            "{}<{}>",
            simple_name,
            &",{}".repeat(relevant_type_params.len() + const_params.len())[1..]
        );

        (
            quote! {
                schemars::_private::alloc::borrow::Cow::Owned(
                    schemars::_private::alloc::format!(#schema_name_fmt #(,schemars::_schemars_maybe_schema_name!(#relevant_type_params))* #(,#const_params)*)
                )
            },
            quote! {
                schemars::_private::alloc::borrow::Cow::Owned(
                    schemars::_private::alloc::format!(
                        ::core::concat!(
                            ::core::module_path!(),
                            "::",
                            #schema_id_fmt
                        )
                        #(,schemars::_schemars_maybe_schema_id!(#relevant_type_params))*
                        #(,#const_params)*
                    )
                )
            },
        )
    }
}
