use crate::attr;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::{Attribute, ExprPath};

#[derive(Debug, Clone, Default)]
pub struct SchemaMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub read_only: bool,
    pub write_only: bool,
    pub default: Option<TokenStream>,
    pub skip_default_if: Option<ExprPath>,
}

impl ToTokens for SchemaMetadata {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let setters = self.make_setters();
        if setters.is_empty() {
            tokens.append(Ident::new("None", Span::call_site()))
        } else {
            tokens.extend(quote! {
                Some({
                    let mut metadata = schemars::schema::Metadata::default();
                    #(#setters)*
                    metadata
                })
            })
        }
    }
}

impl SchemaMetadata {
    pub fn from_doc_attrs(attrs: &[Attribute]) -> SchemaMetadata {
        let (title, description) = attr::get_title_and_desc_from_doc(attrs);
        SchemaMetadata {
            title,
            description,
            ..Default::default()
        }
    }

    pub fn apply_to_schema(&self, schema_expr: TokenStream) -> TokenStream {
        let setters = self.make_setters();

        if setters.is_empty() {
            return schema_expr;
        }

        quote! {
            {
                let mut schema = #schema_expr.into();
                gen.make_extensible(&mut schema);
                let mut metadata = schema.metadata();
                #(#setters)*
                schemars::schema::Schema::Object(schema)
            }
        }
    }

    fn make_setters(self: &SchemaMetadata) -> Vec<TokenStream> {
        let mut setters = Vec::<TokenStream>::new();

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

        match (&self.default, &self.skip_default_if) {
            (Some(default), Some(skip_if)) => setters.push(quote! {
                {
                    let default = #default;
                    if !#skip_if(&default) {
                        metadata.default = schemars::_serde_json::value::to_value(default).ok();
                    }
                }
            }),
            (Some(default), None) => setters.push(quote! {
                metadata.default = schemars::_serde_json::value::to_value(#default).ok();
            }),
            _ => {}
        }

        setters
    }
}
