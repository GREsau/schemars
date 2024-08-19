use crate::_alloc_prelude::*;
use crate::{JsonSchema, Schema, SchemaGenerator};
use serde::Serialize;
use serde_json::{json, map::Entry, Map, Value};

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
#[allow(clippy::needless_pass_by_value)]
pub fn new_externally_tagged_enum_variant(variant: &str, sub_schema: Schema) -> Schema {
    // TODO: this can be optimised by inserting the `sub_schema` as a `Value` rather than
    // using the `json_schema!` macro which borrows and serializes the sub_schema
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

pub fn insert_metadata_property(schema: &mut Schema, key: &str, value: impl Into<Value>) {
    schema.ensure_object().insert(key.to_owned(), value.into());
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
    fn flatten_property(obj1: &mut Map<String, Value>, key: String, value2: Value) {
        match obj1.entry(key) {
            Entry::Vacant(vacant) => match vacant.key().as_str() {
                "additionalProperties" | "unevaluatedProperties" => {
                    if value2 != Value::Bool(false) {
                        vacant.insert(value2);
                    }
                }
                _ => {
                    vacant.insert(value2);
                }
            },
            Entry::Occupied(occupied) => {
                match occupied.key().as_str() {
                    "required" | "allOf" => {
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
                    "additionalProperties" | "unevaluatedProperties" => {
                        // Even if an outer type has `deny_unknown_fields`, unknown fields
                        // may be accepted by the flattened type
                        if occupied.get() == &Value::Bool(false) {
                            *occupied.into_mut() = value2;
                        }
                    }
                    "oneOf" | "anyOf" => {
                        // `OccupiedEntry` currently has no `.remove_entry()` method :(
                        let key = occupied.key().clone();
                        let current = occupied.remove();
                        flatten_property(
                            obj1,
                            "allOf".to_owned(),
                            json!([
                                { &key: current },
                                { key: value2 }
                            ]),
                        );
                    }
                    _ => {
                        // leave the original value as it is (don't modify `schema`)
                    }
                };
            }
        }
    }

    match other.try_to_object() {
        Err(false) => {}
        Err(true) => {
            schema
                .ensure_object()
                .insert("additionalProperties".to_owned(), true.into());
        }
        Ok(obj2) => {
            let obj1 = schema.ensure_object();

            for (key, value2) in obj2 {
                flatten_property(obj1, key, value2);
            }
        }
    }
}
