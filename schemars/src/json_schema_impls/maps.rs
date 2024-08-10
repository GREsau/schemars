use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use std::borrow::Cow;

macro_rules! map_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            V: JsonSchema,
        {
            always_inline!();

            fn schema_name() -> Cow<'static, str> {
                format!("Map_of_{}", V::schema_name()).into()
            }

            fn schema_id() -> Cow<'static, str> {
                format!("Map<{}>", V::schema_id()).into()
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                json_schema!({
                    "type": "object",
                    "additionalProperties": gen.subschema_for::<V>(),
                })
            }
        }
    };
}

map_impl!(<K, V> JsonSchema for std::collections::BTreeMap<K, V>);
map_impl!(<K, V, H> JsonSchema for std::collections::HashMap<K, V, H>);
