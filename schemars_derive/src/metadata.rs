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
    pub crate_name: &'a syn::Path,
    pub examples: &'a [syn::Path],
    pub default: Option<TokenStream>,
}

impl ToTokens for SchemaMetadata<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let setters = self.make_setters();
        let crate_name = self.crate_name;
        if setters.is_empty() {
            tokens.append(Ident::new("None", Span::call_site()))
        } else {
            tokens.extend(quote! {
                Some({
                    let mut metadata = #crate_name::schema::Metadata::default();
                    #(#setters)*
                    metadata
                })
            })
        }
    }
}

impl<'a> SchemaMetadata<'a> {
    // Crate name is separate, because attrs could be for a variant
    // instead of a container, and crate is only applicable to containers.
    pub fn from_attrs(crate_name: &'a syn::Path, attrs: &'a Attrs) -> Self {
        SchemaMetadata {
            title: attrs.title.as_ref().and_then(none_if_empty),
            description: attrs.description.as_ref().and_then(none_if_empty),
            deprecated: attrs.deprecated,
            examples: &attrs.examples,
            crate_name,
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
        let crate_name = self.crate_name;

        if let Some(title) = &self.title {
            setters.push(quote! {
                metadata.title = Some(#title.to_owned());
            });
        }
        if let Some(description) = &self.description {
            setters.push(quote! {
                metadata.description = Some(#description.to_owned());
            });
        }

        if self.deprecated {
            setters.push(quote! {
                metadata.deprecated = true;
            });
        }

        if self.read_only {
            setters.push(quote! {
                metadata.read_only = true;
            });
        }
        if self.write_only {
            setters.push(quote! {
                metadata.write_only = true;
            });
        }

        if !self.examples.is_empty() {
            let examples = self.examples.iter().map(|eg| {
                quote! {
                    #crate_name::_serde_json::value::to_value(#eg())
                }
            });
            setters.push(quote! {
                metadata.examples = vec![#(#examples),*].into_iter().flatten().collect();
            });
        }

        if let Some(default) = &self.default {
            setters.push(quote! {
                metadata.default = #default.and_then(|d| #crate_name::_serde_json::value::to_value(d).ok());
            });
        }

        setters
    }
}

fn none_if_empty<'a>(s: &'a String) -> Option<&'a str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
