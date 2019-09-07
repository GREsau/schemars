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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gen::*;
    use crate::tests::{custom_schema_object_for, schema_for};
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;

    #[test]
    fn schema_for_map_any_value() {
        for bool_schemas in &[BoolSchemas::Enable, BoolSchemas::AdditionalPropertiesOnly] {
            let settings = SchemaSettings {
                bool_schemas: *bool_schemas,
                ..Default::default()
            };
            let schema = custom_schema_object_for::<BTreeMap<String, serde_json::Value>>(settings);
            assert_eq!(
                schema.instance_type,
                Some(SingleOrVec::from(InstanceType::Object))
            );
            assert_eq!(
                schema.extensions.get("additionalProperties"),
                Some(&json!(true))
            );
        }
    }

    #[test]
    fn schema_for_map_any_value_no_bool_schema() {
        let settings = SchemaSettings {
            bool_schemas: BoolSchemas::Disable,
            ..Default::default()
        };
        let schema = custom_schema_object_for::<BTreeMap<String, serde_json::Value>>(settings);
        assert_eq!(
            schema.instance_type,
            Some(SingleOrVec::from(InstanceType::Object))
        );
        assert_eq!(
            schema.extensions.get("additionalProperties"),
            Some(&json!(Schema::Object(Default::default())))
        );
    }

    #[test]
    fn schema_for_map_int_value() {
        for bool_schemas in &[
            BoolSchemas::Enable,
            BoolSchemas::Disable,
            BoolSchemas::AdditionalPropertiesOnly,
        ] {
            let settings = SchemaSettings {
                bool_schemas: *bool_schemas,
                ..Default::default()
            };
            let schema = custom_schema_object_for::<BTreeMap<String, i32>>(settings);
            assert_eq!(
                schema.instance_type,
                Some(SingleOrVec::from(InstanceType::Object))
            );
            assert_eq!(
                schema.extensions.get("additionalProperties"),
                Some(&json!(schema_for::<i32>()))
            );
        }
    }
}
