use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use arrayvec07::{ArrayString, ArrayVec};

// Do not set maxLength on the schema as that describes length in characters, but we only
// know max length in bytes.
forward_impl!((<const CAP: usize> JsonSchema for ArrayString<CAP>) => String);

impl<T, const CAP: usize> JsonSchema for ArrayVec<T, CAP>
where
    T: JsonSchema,
{
    no_ref_schema!();

    fn schema_name() -> String {
        format!("Array_up_to_size_{}_of_{}", CAP, T::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "array",
            "items": gen.subschema_for::<T>(),
            "maxItems": CAP
        })
    }
}
