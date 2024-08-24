use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Attribute;

pub fn get_doc(attrs: &[Attribute]) -> Option<syn::Expr> {
    let joiner = quote! {, "\n",};
    let mut macro_args: TokenStream = TokenStream::new();

    for nv in attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| a.meta.require_name_value().ok())
    {
        if !macro_args.is_empty() {
            macro_args.extend(joiner.clone());
        }
        macro_args.extend(nv.value.to_token_stream());
    }

    if macro_args.is_empty() {
        None
    } else {
        Some(parse_quote!(::core::concat!(#macro_args)))
    }
}
