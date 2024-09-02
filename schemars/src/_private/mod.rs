use crate::_alloc_prelude::*;
use crate::transform::transform_immediate_subschemas;
use crate::{JsonSchema, Schema, SchemaGenerator};
use serde::Serialize;
use serde_json::{json, map::Entry, Map, Value};

mod regex_syntax;
mod rustdoc;

pub extern crate alloc;
pub extern crate serde_json;

pub use rustdoc::get_title_and_description;

// Helper for generating schemas for flattened `Option` fields.
pub fn json_schema_for_flatten<T: ?Sized + JsonSchema>(
    generator: &mut SchemaGenerator,
    required: bool,
) -> Schema {
    let mut schema = T::_schemars_private_non_optional_json_schema(generator);

    if T::_schemars_private_is_option() && !required {
        schema.remove("required");
    }

    // Always allow aditional/unevaluated properties, because the outer struct determines
    // whether it denies unknown fields.
    allow_unknown_properties(&mut schema);

    schema
}

fn allow_unknown_properties(schema: &mut Schema) {
    if schema.get("additionalProperties").and_then(Value::as_bool) == Some(false) {
        schema.remove("additionalProperties");
    }
    if schema.get("unevaluatedProperties").and_then(Value::as_bool) == Some(false) {
        schema.remove("unevaluatedProperties");
    }

    transform_immediate_subschemas(&mut allow_unknown_properties, schema);
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

pub fn insert_object_property(
    schema: &mut Schema,
    key: &str,
    is_optional: bool,
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

    if !is_optional {
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

pub fn insert_metadata_property_if_nonempty(
    schema: &mut Schema,
    key: &str,
    value: impl Into<String>,
) {
    let value: String = value.into();
    if !value.is_empty() {
        insert_metadata_property(schema, key, value);
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

pub fn must_contain(schema: &mut Schema, substring: &str) {
    let escaped = regex_syntax::escape(substring);
    insert_validation_property(schema, "string", "pattern", escaped);
}

pub fn apply_inner_validation(schema: &mut Schema, f: fn(&mut Schema) -> ()) {
    if let Some(inner_schema) = schema
        .as_object_mut()
        .and_then(|o| o.get_mut("items"))
        .and_then(|i| i.try_into().ok())
    {
        f(inner_schema);
    }
}

pub fn flatten(schema: &mut Schema, other: Schema) {
    fn flatten_property(obj1: &mut Map<String, Value>, key: String, value2: Value) {
        match obj1.entry(key) {
            Entry::Vacant(vacant) => {
                vacant.insert(value2);
            }
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
                    "oneOf" | "anyOf" => {
                        let (key, current) = occupied.remove_entry();
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
            if let Some(obj) = schema.as_object_mut() {
                if !obj.contains_key("additionalProperties")
                    && !obj.contains_key("unevaluatedProperties")
                {
                    let key = if contains_immediate_subschema(obj) {
                        "unevaluatedProperties"
                    } else {
                        "additionalProperties"
                    };
                    obj.insert(key.to_owned(), true.into());
                }
            }
        }
        Ok(mut obj2) => {
            let obj1 = schema.ensure_object();

            // For complex merges, replace `additionalProperties` with `unevaluatedProperties`
            // which usually "works out better".
            normalise_additional_unevaluated_properties(obj1, &obj2);
            normalise_additional_unevaluated_properties(&mut obj2, obj1);

            for (key, value2) in obj2 {
                flatten_property(obj1, key, value2);
            }
        }
    }
}

fn normalise_additional_unevaluated_properties(
    schema_obj1: &mut Map<String, Value>,
    schema_obj2: &Map<String, Value>,
) {
    if schema_obj1.contains_key("additionalProperties")
        && (schema_obj2.contains_key("unevaluatedProperties")
            || contains_immediate_subschema(schema_obj2))
    {
        let ap = schema_obj1.remove("additionalProperties");
        schema_obj1.insert("unevaluatedProperties".to_owned(), ap.into());
    }
}

fn contains_immediate_subschema(schema_obj: &Map<String, Value>) -> bool {
    ["if", "then", "else", "allOf", "anyOf", "oneOf", "$ref"]
        .into_iter()
        .any(|k| schema_obj.contains_key(k))
}
