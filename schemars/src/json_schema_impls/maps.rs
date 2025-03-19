use crate::_alloc_prelude::*;
use crate::{json_schema, JsonSchema, Schema, SchemaGenerator};
use alloc::borrow::Cow;

macro_rules! map_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            V: JsonSchema,
            K: JsonSchema,
        {
            always_inline!();

            fn schema_name() -> Cow<'static, str> {
                format!("Map_of_{}", V::schema_name()).into()
            }

            fn schema_id() -> Cow<'static, str> {
                format!("Map<{}>", V::schema_id()).into()
            }

            fn json_schema(generator: &mut SchemaGenerator) -> Schema {
                let key_schema = generator.subschema_for::<K>();

                if let Some(key_schema) = key_schema.as_object() {
                    if key_schema.get("$ref").is_some() {
                        return json_schema!({
                            "type": "object",
                            "additionalProperties": generator.subschema_for::<V>(),
                            "propertyNames": generator.subschema_for::<K>(),
                        })
                    }
                }

                json_schema!({
                    "type": "object",
                    "additionalProperties": generator.subschema_for::<V>(),
                })
            }
        }
    };
}

map_impl!(<K, V> JsonSchema for alloc::collections::BTreeMap<K, V>);

#[cfg(feature = "std")]
map_impl!(<K, V, H> JsonSchema for std::collections::HashMap<K, V, H>);