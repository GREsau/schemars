use crate::gen::{BoolSchemas, SchemaGenerator};
use crate::schema::*;
use crate::{JsonSchema, Map, Result};
use serde_json::json;

macro_rules! map_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            K: Into<String>,
            V: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> String {
                format!("Map_Of_{}", V::schema_name())
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Result {
                let subschema = gen.subschema_for::<V>()?;
                let json_schema_bool = gen.settings().bool_schemas == BoolSchemas::AdditionalPropertiesOnly
                    && subschema == gen.schema_for_any();
                let mut extensions = Map::new();
                extensions.insert(
                    "additionalProperties".to_owned(),
                    if json_schema_bool {
                        json!(true)
                    } else {
                        json!(subschema)
                    }
                );
                Ok(SchemaObject {
                    instance_type: Some(InstanceType::Object.into()),
                    extensions,
                    ..Default::default()
                }.into())
            }
        }
    };
}

map_impl!(<K: Ord, V> JsonSchema for std::collections::BTreeMap<K, V>);
map_impl!(<K: Eq + core::hash::Hash, V, H: core::hash::BuildHasher> JsonSchema for std::collections::HashMap<K, V, H>);
