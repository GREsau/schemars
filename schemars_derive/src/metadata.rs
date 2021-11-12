use proc_macro2::TokenStream;

#[derive(Debug, Clone)]
pub struct SchemaMetadata<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub deprecated: bool,
    pub read_only: bool,
    pub write_only: bool,
    pub examples: &'a [syn::Path],
    pub example_sets: &'a [syn::Path],
    pub default: Option<TokenStream>,
}

impl<'a> SchemaMetadata<'a> {
    pub fn apply_to_schema(&self, schema_expr: &mut TokenStream) {
        let setters = self.make_setters();
        if !setters.is_empty() {
            *schema_expr = quote! {
                schemars::_private::apply_metadata(#schema_expr, schemars::schema::Metadata {
                    #(#setters)*
                    ..Default::default()
                })
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

        let examples = self.examples.iter().map(|eg| {
            quote! {
                schemars::_serde_json::value::to_value(#eg())
            }
        });

        let example_sets = self.example_sets.iter().map(|eg| {
            quote! {
                #eg().into_iter().map(schemars::_serde_json::value::to_value)
            }
        });

        // generate different code when one of the collections is empty,
        // otherwise rustc has trouble inferring the element type for the empty vec
        match (!self.example_sets.is_empty(), !self.examples.is_empty()) {
            (true, true) => setters.push(quote! {
                examples: vec![#(#example_sets),*]
                    .into_iter()
                    .flatten() // example sets to single examples
                    .chain(vec![#(#examples),*])
                    .flatten() // remove err variants from to_value results
                    .collect(),
            }),
            (false, true) => setters.push(quote! {
                examples: vec![#(#examples),*]
                    .into_iter()
                    .flatten() // remove err variants from to_value results
                    .collect(),
            }),
            (true, false) => setters.push(quote! {
                examples: vec![#(#example_sets),*]
                    .into_iter()
                    .flatten() // example sets to single examples
                    .flatten() // remove err variants from to_value results
                    .collect(),
            }),
            (false, false) => {}
        }

        if let Some(default) = &self.default {
            setters.push(quote! {
                default: #default.and_then(|d| schemars::_serde_json::value::to_value(d).ok()),
            });
        }

        setters
    }
}
