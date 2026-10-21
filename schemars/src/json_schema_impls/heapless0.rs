use crate::SchemaGenerator;
use crate::_alloc_prelude::*;
use crate::{json_schema, JsonSchema, Schema};
use heapless0::LinearMap;

// Do not set maxLength on the schema as that describes length in characters, but we only
// know max length in bytes.
forward_impl!((<const N: usize> JsonSchema for heapless0::String<N>) => String);

impl<T, const N: usize> JsonSchema for heapless0::Vec<T, N>
where
    T: JsonSchema,
{
    inline_schema!();

    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        format!("Array_up_to_size_{}_of_{}", N, T::schema_name()).into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "array",
            "items": generator.subschema_for::<T>(),
            "maxItems": N
        })
    }
}

impl<K, V, const N: usize> JsonSchema for LinearMap<K, V, N>
where
    K: JsonSchema,
    V: JsonSchema,
{
    inline_schema!();

    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        format!("Map_up_to_size_{}_from_{}_to_{}", N, K::schema_name(), V::schema_name()).into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "object",
            "additionalProperties": generator.subschema_for::<V>(),
            "maxProperties": N
        })
    }
}
