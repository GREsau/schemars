use crate::doc_attrs;
use proc_macro2::TokenStream;
use syn::Attribute;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SchemaMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub read_only: bool,
    pub write_only: bool,
}

pub fn set_metadata_on_schema_from_docs(
    schema_expr: TokenStream,
    attrs: &[Attribute],
) -> TokenStream {
    let metadata = get_metadata_from_docs(attrs);
    set_metadata_on_schema(schema_expr, &metadata)
}

pub fn get_metadata_from_docs(attrs: &[Attribute]) -> SchemaMetadata {
    let (title, description) = doc_attrs::get_title_and_desc_from_docs(attrs);
    SchemaMetadata {
        title,
        description,
        ..Default::default()
    }
}

pub fn set_metadata_on_schema(schema_expr: TokenStream, metadata: &SchemaMetadata) -> TokenStream {
    let mut setters = Vec::<TokenStream>::new();

    if let Some(title) = &metadata.title {
        setters.push(quote! {
            metadata.title = Some(#title.to_owned());
        })
    }
    if let Some(description) = &metadata.description {
        setters.push(quote! {
            metadata.description = Some(#description.to_owned());
        })
    }
    if metadata.read_only {
        setters.push(quote! {
            metadata.read_only = true;
        })
    }
    if metadata.write_only {
        setters.push(quote! {
            metadata.write_only = true;
        })
    }

    if setters.is_empty() {
        return schema_expr;
    }

    quote! {
        {
            let schema = #schema_expr.into();
            let mut schema_obj = gen.objectify(schema);
            let mut metadata = schema_obj.metadata();
            #(#setters)*
            schemars::schema::Schema::Object(schema_obj)
        }
    }
}
