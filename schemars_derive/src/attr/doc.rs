use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::Attribute;

pub fn get_doc(attrs: &[Attribute]) -> Option<syn::Expr> {
    let mut macro_args: TokenStream = TokenStream::new();

    let lines = attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .flat_map(|a| a.meta.require_name_value())
        .map(|m| &m.value);
    macro_args.append_separated(lines, quote!(, "\n",));

    if macro_args.is_empty() {
        None
    } else {
        Some(parse_quote!(::core::concat!(#macro_args)))
    }
}
