use crate::flatten::Merge;
use crate::gen::SchemaGenerator;
use crate::schema::{Metadata, Schema, SchemaObject};
use crate::JsonSchema;

// Helper for generating schemas for flattened `Option` fields.
pub fn json_schema_for_flatten<T: ?Sized + JsonSchema>(
    gen: &mut SchemaGenerator,
    required: bool,
) -> Schema {
    let mut schema = T::_schemars_private_non_optional_json_schema(gen);

    if T::_schemars_private_is_option() && !required {
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

pub fn apply_metadata(schema: Schema, metadata: Metadata) -> Schema {
    if metadata == Metadata::default() {
        schema
    } else {
        let mut schema_obj = schema.into_object();
        schema_obj.metadata = Some(Box::new(metadata)).merge(schema_obj.metadata);
        Schema::Object(schema_obj)
    }
}
