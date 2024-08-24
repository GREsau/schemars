use proc_macro2::TokenStream;
use syn::{spanned::Spanned, Expr};

#[derive(Debug, Clone)]
pub struct SchemaMetadata<'a> {
    pub title: Option<&'a Expr>,
    pub description: Option<&'a Expr>,
    pub doc: Option<&'a Expr>,
    pub deprecated: bool,
    pub read_only: bool,
    pub write_only: bool,
    pub examples: &'a [syn::Path],
    pub default: Option<TokenStream>,
    pub extensions: &'a [(String, TokenStream)],
    pub transforms: &'a [Expr],
}

impl<'a> SchemaMetadata<'a> {
    pub fn apply_to_schema(&self, schema_expr: &mut TokenStream) {
        let setters = self.make_setters();
        if !setters.is_empty() || !self.transforms.is_empty() {
            let apply_transforms = self.transforms.iter().map(|t| {
                quote_spanned! {t.span()=>
                    schemars::transform::Transform::transform(&mut #t, &mut schema);
                }
            });
            *schema_expr = quote! {{
                let mut schema = #schema_expr;
                #(#setters)*
                #(#apply_transforms)*
                schema
            }}
        }
    }

    fn make_setters(&self) -> Vec<TokenStream> {
        let mut setters = Vec::<TokenStream>::new();

        if let Some(doc) = &self.doc {
            if self.title.is_none() || self.description.is_none() {
                setters.push(quote!{
                    const title_and_description: (&str, &str) = schemars::_private::get_title_and_description(#doc);
                });
            }
        }
        if let Some(title) = &self.title {
            setters.push(quote! {
                schemars::_private::insert_metadata_property_if_nonempty(&mut schema, "title", #title);
            });
        } else if self.doc.is_some() {
            setters.push(quote! {
                schemars::_private::insert_metadata_property_if_nonempty(&mut schema, "title", title_and_description.0);
            });
        }
        if let Some(description) = &self.description {
            setters.push(quote! {
                schemars::_private::insert_metadata_property_if_nonempty(&mut schema, "description", #description);
            });
        } else if self.doc.is_some() {
            setters.push(quote! {
                schemars::_private::insert_metadata_property_if_nonempty(&mut schema, "description", title_and_description.1);
            });
        }

        if self.deprecated {
            setters.push(quote! {
                schemars::_private::insert_metadata_property(&mut schema, "deprecated", true);
            });
        }

        if self.read_only {
            setters.push(quote! {
                schemars::_private::insert_metadata_property(&mut schema, "readOnly", true);
            });
        }
        if self.write_only {
            setters.push(quote! {
                schemars::_private::insert_metadata_property(&mut schema, "writeOnly", true);
            });
        }

        if !self.examples.is_empty() {
            let examples = self.examples.iter().map(|eg| {
                quote! {
                    schemars::_serde_json::value::to_value(#eg())
                }
            });
            setters.push(quote! {
                schemars::_private::insert_metadata_property(&mut schema, "examples", schemars::_serde_json::Value::Array([#(#examples),*].into_iter().flatten().collect()));
            });
        }

        if let Some(default) = &self.default {
            setters.push(quote! {
                if let Some(default) = #default.and_then(|d| schemars::_schemars_maybe_to_value!(d)) {
                    schemars::_private::insert_metadata_property(&mut schema, "default", default);
                }
            });
        }

        for (k, v) in self.extensions {
            setters.push(quote! {
                schemars::_private::insert_metadata_property(&mut schema, #k, schemars::_serde_json::json!(#v));
            });
        }

        setters
    }
}
