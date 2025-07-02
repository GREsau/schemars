use crate::_alloc_prelude::*;
use crate::{json_schema, JsonSchema, Schema, SchemaGenerator};
use alloc::borrow::Cow;
use alloc::collections::{BTreeMap, BTreeSet};
use serde_json::{Map, Value};

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
enum IntegerSupport {
    None,
    Unsigned,
    Signed,
}

impl<K, V> JsonSchema for BTreeMap<K, V>
where
    K: JsonSchema,
    V: JsonSchema,
{
    inline_schema!();

    fn schema_name() -> Cow<'static, str> {
        Cow::Owned(if K::schema_id() == <str>::schema_id() {
            format!("Map_of_{}", V::schema_name())
        } else {
            format!("Map_from_{}_to_{}", K::schema_name(), V::schema_name())
        })
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Owned(if K::schema_id() == <str>::schema_id() {
            format!("Map<{}>", V::schema_id())
        } else {
            format!("Map<{}, {}>", K::schema_id(), V::schema_id())
        })
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let key_schema = K::json_schema(generator);
        let value_schema = generator.subschema_for::<V>();

        let mut map_schema = json_schema!({
            "type": "object",
        });

        let Some(mut options) = key_schema
            .get("anyOf")
            .and_then(Value::as_array)
            .and_then(|a| a.iter().map(Value::as_object).collect::<Option<Vec<_>>>())
            .or_else(|| Some(vec![key_schema.as_object()?]))
        else {
            return json_schema!({
                "additionalProperties": value_schema,
                "type": "object",
            });
        };

        // Handle refs
        let prefix = format!("#{}/", generator.definitions_path_stripped());
        for option in &mut options {
            if let Some(d) = option
                .get("$ref")
                .and_then(Value::as_str)
                .and_then(|r| r.strip_prefix(&prefix))
                .and_then(|r| generator.definitions().get(r))
                .and_then(Value::as_object)
            {
                *option = d;
            }
        }

        let mut additional_properties = false;
        let mut support_integers = IntegerSupport::None;
        let mut patterns = BTreeSet::new();
        let mut properties = BTreeSet::new();
        for option in options {
            let key_pattern = option.get("pattern").and_then(Value::as_str);
            let key_enum = option
                .get("enum")
                .and_then(Value::as_array)
                .and_then(|a| a.iter().map(Value::as_str).collect::<Option<Vec<_>>>());
            let key_type = option.get("type").and_then(Value::as_str);
            let key_minimum = option.get("minimum").and_then(Value::as_u64);

            match (key_pattern, key_enum, key_type) {
                (Some(pattern), _, Some("string")) => {
                    patterns.insert(pattern);
                }
                (None, Some(enum_values), Some("string")) => {
                    for value in enum_values {
                        properties.insert(value);
                    }
                }
                (_, _, Some("integer")) if key_minimum == Some(0) => {
                    support_integers = support_integers.max(IntegerSupport::Unsigned);
                }
                (_, _, Some("integer")) => {
                    support_integers = support_integers.max(IntegerSupport::Signed);
                }
                _ => {
                    additional_properties = true;
                }
            }
        }

        if additional_properties {
            map_schema.insert(
                "additionalProperties".to_owned(),
                value_schema.clone().to_value(),
            );
        } else {
            map_schema.insert("additionalProperties".to_owned(), Value::Bool(false));
        }

        match support_integers {
            IntegerSupport::None => {}
            IntegerSupport::Unsigned => {
                patterns.insert(r"^\d+$");
            }
            IntegerSupport::Signed => {
                patterns.insert(r"^-?\d+$");
            }
        }

        if !patterns.is_empty() {
            let mut patterns_map = Map::new();

            for pattern in patterns {
                patterns_map.insert(pattern.to_owned(), value_schema.clone().to_value());
            }

            map_schema.insert("patternProperties".to_owned(), Value::Object(patterns_map));
        }

        if !properties.is_empty() {
            let mut properties_map = Map::new();

            for property in properties {
                properties_map.insert(property.to_owned(), value_schema.clone().to_value());
            }

            map_schema.insert("properties".to_owned(), Value::Object(properties_map));
        }

        map_schema
    }
}

#[cfg(feature = "std")]
forward_impl!((<K: JsonSchema, V: JsonSchema, H> JsonSchema for std::collections::HashMap<K, V, H>) => BTreeMap<K, V>);
