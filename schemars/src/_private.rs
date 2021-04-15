use crate::flatten::Merge;
use crate::gen::SchemaGenerator;
use crate::schema::{Metadata, Schema, SchemaObject};
use crate::JsonSchema;

// Helper for generating schemas for flattened `Option` fields.
pub fn json_schema_for_flatten<T: ?Sized + JsonSchema>(
    gen: &mut SchemaGenerator,
    required: Option<bool>,
) -> Schema {
    let mut schema = T::_schemars_private_non_optional_json_schema(gen);

    let required = required.unwrap_or_else(|| !T::_schemars_private_is_option());
    if !required {
        if let Schema::Object(SchemaObject {
            object: Some(ref mut object_validation),
            ..
        }) = schema
        {
            object_validation.required.clear();
        }
    }

    schema
}

// Helper for generating schemas for `Option` fields.
pub fn add_schema_as_property<T: ?Sized + JsonSchema>(
    gen: &mut SchemaGenerator,
    parent: &mut SchemaObject,
    name: String,
    metadata: Option<Metadata>,
    required: Option<bool>,
) {
    let is_type_option = T::_schemars_private_is_option();
    let required = required.unwrap_or(!is_type_option);

    let mut schema = if required && is_type_option {
        T::_schemars_private_non_optional_json_schema(gen)
    } else {
        gen.subschema_for::<T>()
    };

    schema = apply_metadata(schema, metadata);

    let object = parent.object();
    if required {
        object.required.insert(name.clone());
    }
    object.properties.insert(name, schema);
}

pub fn apply_metadata(schema: Schema, metadata: Option<Metadata>) -> Schema {
    match metadata {
        None => schema,
        Some(ref metadata) if *metadata == Metadata::default() => schema,
        Some(metadata) => {
            let mut schema_obj = schema.into_object();
            schema_obj.metadata = Some(Box::new(metadata)).merge(schema_obj.metadata);
            Schema::Object(schema_obj)
        }
    }
}
