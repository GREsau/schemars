use proc_macro2::TokenStream;

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

impl<'a> SchemaMetadata<'a> {
    pub fn apply_to_schema(&self, schema_expr: &mut TokenStream) {
        let setters = self.make_setters();
        if !setters.is_empty() {
            *schema_expr = quote! {{
                let mut schema = #schema_expr;
                let obj = schema.ensure_object();
                #(#setters)*
                schema
            }}
        }
    }

    fn make_setters(&self) -> Vec<TokenStream> {
        let mut setters = Vec::<TokenStream>::new();

        if let Some(title) = &self.title {
            setters.push(quote! {
                obj.insert("title".to_owned(), #title.into());
            });
        }
        if let Some(description) = &self.description {
            setters.push(quote! {
                obj.insert("description".to_owned(), #description.into());
            });
        }

        if self.deprecated {
            setters.push(quote! {
                obj.insert("deprecated".to_owned(), true.into());
            });
        }

        if self.read_only {
            setters.push(quote! {
                obj.insert("readOnly".to_owned(), true.into());
            });
        }
        if self.write_only {
            setters.push(quote! {
                obj.insert("writeOnly".to_owned(), true.into());
            });
        }

        if !self.examples.is_empty() {
            let examples = self.examples.iter().map(|eg| {
                quote! {
                    schemars::_serde_json::value::to_value(#eg())
                }
            });
            setters.push(quote! {
                obj.insert("examples".to_owned(), schemars::_serde_json::Value::Array([#(#examples),*].into_iter().flatten().collect()));
            });
        }

        if let Some(default) = &self.default {
            setters.push(quote! {
                if let Some(default) = #default.and_then(|d| schemars::_schemars_maybe_to_value!(d)) {
                    obj.insert("default".to_owned(), default);
                }
            });
        }

        setters
    }
}
