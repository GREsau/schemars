use crate::attr;
use proc_macro2::TokenStream;
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

pub fn set_metadata_on_schema_from_docs(
    schema_expr: TokenStream,
    attrs: &[Attribute],
) -> TokenStream {
    let metadata = get_metadata_from_docs(attrs);
    set_metadata_on_schema(schema_expr, &metadata)
}

pub fn get_metadata_from_docs(attrs: &[Attribute]) -> SchemaMetadata {
    let (title, description) = attr::get_title_and_desc_from_doc(attrs);
    SchemaMetadata {
        title,
        description,
        ..Default::default()
    }
}

pub fn quote_metadata(metadata: &SchemaMetadata) -> TokenStream {
    let setters = make_metadata_setters(metadata);

    if setters.is_empty() {
        return quote!(None);
    }

    quote! {
        Some({
            let mut metadata = schemars::schema::Metadata::default();
            #(#setters)*
            metadata
        })
    }
}

pub fn set_metadata_on_schema(schema_expr: TokenStream, metadata: &SchemaMetadata) -> TokenStream {
    let setters = make_metadata_setters(metadata);

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

fn make_metadata_setters(metadata: &SchemaMetadata) -> Vec<TokenStream> {
    let mut setters = Vec::<TokenStream>::new();

    if let Some(title) = &metadata.title {
        setters.push(quote! {
            metadata.title = Some(#title.to_owned());
        });
    }
    if let Some(description) = &metadata.description {
        setters.push(quote! {
            metadata.description = Some(#description.to_owned());
        });
    }

    if metadata.read_only {
        setters.push(quote! {
            metadata.read_only = true;
        });
    }
    if metadata.write_only {
        setters.push(quote! {
            metadata.write_only = true;
        });
    }

    match (&metadata.default, &metadata.skip_default_if) {
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
