use crate::gen::SchemaGenerator;
use crate::JsonSchema;
use crate::Schema;
use serde::Serialize;
use serde_json::json;
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

/// Create a schema for a unit enum
pub fn new_unit_enum(variant: &str) -> Schema {
    // TODO switch from single-valued "enum" to "const"
    json_schema!({
        "type": "string",
        "enum": [variant],
    })
}

/// Create a schema for an externally tagged enum
pub fn new_externally_tagged_enum(variant: &str, sub_schema: Schema) -> Schema {
    json_schema!({
        "type": "object",
        "properties": {
            variant: sub_schema
        },
        "required": [variant],
        "additionalProperties": false,
    })
}

pub fn apply_internal_enum_tag(
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
                // TODO switch from single-valued "enum" to "const"
                "enum": [variant]
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

/// Create a schema for an internally tagged enum
pub fn new_internally_tagged_enum(
    tag_name: &str,
    variant: &str,
    deny_unknown_fields: bool,
) -> Schema {
    // TODO switch from single-valued "enum" to "const"
    let mut schema = json_schema!({
        "type": "object",
        "properties": {
            tag_name: {
                "type": "string",
                "enum": [variant],
            }
        },
        "required": [tag_name],
    });

    if deny_unknown_fields {
        schema
            .as_object_mut()
            .unwrap()
            .insert("additionalProperties".into(), false.into());
    }

    schema
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

    if required || !(has_default || T::_schemars_private_is_option()) {
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
