use crate::attr;
use attr::Attrs;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};

#[derive(Debug, Clone)]
pub struct SchemaMetadata<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub deprecated: bool,
    pub read_only: bool,
    pub write_only: bool,
    pub examples: &'a [syn::Path],
    pub default: Option<TokenStream>,
}

impl ToTokens for SchemaMetadata<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let setters = self.make_setters();
        if setters.is_empty() {
            tokens.append(Ident::new("None", Span::call_site()))
        } else {
            tokens.extend(quote! {
                Some({
                    schemars::schema::Metadata {
                        #(#setters)*
                        ..Default::default()
                    }
                })
            })
        }
    }
}

impl<'a> SchemaMetadata<'a> {
    pub fn from_attrs(attrs: &'a Attrs) -> Self {
        SchemaMetadata {
            title: attrs.title.as_ref().and_then(none_if_empty),
            description: attrs.description.as_ref().and_then(none_if_empty),
            deprecated: attrs.deprecated,
            examples: &attrs.examples,
            read_only: false,
            write_only: false,
            default: None,
        }
    }

    pub fn apply_to_schema(&self, schema_expr: TokenStream) -> TokenStream {
        quote! {
            {
                let schema = #schema_expr;
                gen.apply_metadata(schema, #self)
            }
        }
    }

    fn make_setters(&self) -> Vec<TokenStream> {
        let mut setters = Vec::<TokenStream>::new();

        if let Some(title) = &self.title {
            setters.push(quote! {
                title: Some(#title.to_owned()),
            });
        }
        if let Some(description) = &self.description {
            setters.push(quote! {
                description: Some(#description.to_owned()),
            });
        }

        if self.deprecated {
            setters.push(quote! {
                deprecated: true,
            });
        }

        if self.read_only {
            setters.push(quote! {
                read_only: true,
            });
        }
        if self.write_only {
            setters.push(quote! {
                write_only: true,
            });
        }

        if !self.examples.is_empty() {
            let examples = self.examples.iter().map(|eg| {
                quote! {
                    schemars::_serde_json::value::to_value(#eg())
                }
            });
            setters.push(quote! {
                examples: vec![#(#examples),*].into_iter().flatten().collect(),
            });
        }

        if let Some(default) = &self.default {
            setters.push(quote! {
                default: #default.and_then(|d| schemars::_serde_json::value::to_value(d).ok()),
            });
        }

        setters
    }
}

#[allow(clippy::ptr_arg)]
fn none_if_empty(s: &String) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
