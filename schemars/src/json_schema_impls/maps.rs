use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use std::borrow::Cow;

macro_rules! map_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            V: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> String {
                format!("Map_of_{}", V::schema_name())
            }

            fn schema_id() -> Cow<'static, str> {
                Cow::Owned(format!("Map<{}>", V::schema_id()))
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
