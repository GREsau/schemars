use crate::gen::SchemaGenerator;
use crate::JsonSchema;
use crate::Schema;
use serde::Serialize;
use serde_json::json;
use serde_json::map::Entry;
use serde_json::Map;
use serde_json::Value;

// Helper for generating schemas for flattened `Option` fields.
pub fn json_schema_for_flatten<T: ?Sized + JsonSchema>(
    gen: &mut SchemaGenerator,
    required: bool,
) -> Schema {
    let mut schema = T::_schemars_private_non_optional_json_schema(gen);

    if T::_schemars_private_is_option() && !required {
        if let Some(object) = schema.as_object_mut() {
            object.remove("required");
        }
    }

    schema
}

/// Hack to simulate specialization:
/// `MaybeSerializeWrapper(x).maybe_to_value()` will resolve to either
/// - The inherent method `MaybeSerializeWrapper::maybe_to_value(...)` if x is `Serialize`
/// - The trait method `NoSerialize::maybe_to_value(...)` from the blanket impl otherwise
#[doc(hidden)]
#[macro_export]
macro_rules! _schemars_maybe_to_value {
    ($expression:expr) => {{
        #[allow(unused_imports)]
        use $crate::_private::{MaybeSerializeWrapper, NoSerialize as _};

        MaybeSerializeWrapper($expression).maybe_to_value()
    }};
}

pub struct MaybeSerializeWrapper<T>(pub T);

pub trait NoSerialize: Sized {
    fn maybe_to_value(self) -> Option<Value> {
        None
    }
}

impl<T> NoSerialize for T {}

impl<T: Serialize> MaybeSerializeWrapper<T> {
    pub fn maybe_to_value(self) -> Option<Value> {
        serde_json::value::to_value(self.0).ok()
    }
}

/// Create a schema for a unit enum variant
pub fn new_unit_enum_variant(variant: &str) -> Schema {
    json_schema!({
        "type": "string",
        "const": variant,
    })
}

/// Create a schema for an externally tagged enum variant
pub fn new_externally_tagged_enum_variant(variant: &str, sub_schema: Schema) -> Schema {
    json_schema!({
        "type": "object",
        "properties": {
            variant: sub_schema
        },
        "required": [variant],
        "additionalProperties": false,
    })
}

/// Update a schema for an internally tagged enum variant
pub fn apply_internal_enum_variant_tag(
    schema: &mut Schema,
    tag_name: &str,
    variant: &str,
    deny_unknown_fields: bool,
) {
    let obj = schema.ensure_object();
    let is_unit = obj.get("type").and_then(|t| t.as_str()) == Some("null");

    obj.insert("type".to_owned(), "object".into());

    if let Some(properties) = obj
        .entry("properties")
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
    {
        properties.insert(
            tag_name.to_string(),
            json!({
                "type": "string",
                "const": variant
            }),
        );
    }

    if let Some(required) = obj
        .entry("required")
        .or_insert(Value::Array(Vec::new()))
        .as_array_mut()
    {
        required.insert(0, tag_name.into());
    }

    if deny_unknown_fields && is_unit {
        obj.entry("additionalProperties").or_insert(false.into());
    }
}

pub fn insert_object_property<T: ?Sized + JsonSchema>(
    schema: &mut Schema,
    key: &str,
    has_default: bool,
    required: bool,
    sub_schema: Schema,
) {
    let obj = schema.ensure_object();
    if let Some(properties) = obj
        .entry("properties")
        .or_insert(Value::Object(Map::new()))
        .as_object_mut()
    {
        properties.insert(key.to_owned(), sub_schema.into());
    }

    if !has_default && (required || !T::_schemars_private_is_option()) {
        if let Some(req) = obj
            .entry("required")
            .or_insert(Value::Array(Vec::new()))
            .as_array_mut()
        {
            req.push(key.into());
        }
    }
}

pub fn insert_validation_property(
    schema: &mut Schema,
    required_type: &str,
    key: &str,
    value: impl Into<Value>,
) {
    if schema.has_type(required_type) || (required_type == "number" && schema.has_type("integer")) {
        schema.ensure_object().insert(key.to_owned(), value.into());
    }
}

pub fn append_required(schema: &mut Schema, key: &str) {
    if schema.has_type("object") {
        if let Value::Array(array) = schema
            .ensure_object()
            .entry("required")
            .or_insert(Value::Array(Vec::new()))
        {
            let value = Value::from(key);
            if !array.contains(&value) {
                array.push(value);
            }
        }
    }
}

pub fn apply_inner_validation(schema: &mut Schema, f: fn(&mut Schema) -> ()) {
    if let Some(inner_schema) = schema
        .as_object_mut()
        .and_then(|o| o.get_mut("items"))
        .and_then(|i| <&mut Schema>::try_from(i).ok())
    {
        f(inner_schema);
    }
}

pub fn flatten(schema: &mut Schema, other: Schema) {
    if let Value::Object(obj2) = other.to_value() {
        let obj1 = schema.ensure_object();

        for (key, value2) in obj2 {
            match obj1.entry(key) {
                Entry::Vacant(vacant) => {
                    vacant.insert(value2);
                }
                Entry::Occupied(mut occupied) => {
                    match occupied.key().as_str() {
                        // This special "type" handling can probably be removed once the enum variant `with`/`schema_with` behaviour is fixed
                        "type" => match (occupied.get_mut(), value2) {
                            (Value::Array(a1), Value::Array(mut a2)) => {
                                a2.retain(|v2| !a1.contains(v2));
                                a1.extend(a2);
                            }
                            (v1, Value::Array(mut a2)) => {
                                if !a2.contains(v1) {
                                    a2.push(std::mem::take(v1));
                                    *occupied.get_mut() = Value::Array(a2);
                                }
                            }
                            (Value::Array(a1), v2) => {
                                if !a1.contains(&v2) {
                                    a1.push(v2);
                                }
                            }
                            (v1, v2) => {
                                if v1 != &v2 {
                                    *occupied.get_mut() =
                                        Value::Array(vec![std::mem::take(v1), v2]);
                                }
                            }
                        },
                        "required" => {
                            if let Value::Array(a1) = occupied.into_mut() {
                                if let Value::Array(a2) = value2 {
                                    a1.extend(a2);
                                }
                            }
                        }
                        "properties" | "patternProperties" => {
                            if let Value::Object(o1) = occupied.into_mut() {
                                if let Value::Object(o2) = value2 {
                                    o1.extend(o2);
                                }
                            }
                        }
                        _ => {
                            // leave the original value as it is (don't modify `schema`)
                        }
                    };
                }
            }
        }
    }
}
