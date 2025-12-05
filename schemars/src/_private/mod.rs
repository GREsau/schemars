use crate::_alloc_prelude::*;
use crate::transform::{transform_immediate_subschemas, Transform};
use crate::{JsonSchema, Schema, SchemaGenerator};
use alloc::borrow::Cow;
use serde::Serialize;
use serde_json::{json, map::Entry, Map, Value};

mod regex_syntax;
mod rustdoc;

pub extern crate alloc;
pub extern crate serde_json;

pub use rustdoc::get_title_and_description;

pub fn json_schema_for_internally_tagged_enum_newtype_variant<T: ?Sized + JsonSchema>(
    generator: &mut SchemaGenerator,
) -> Schema {
    let mut schema = T::json_schema(generator);

    // Inline the newtype's inner schema if any of:
    // - The type specifies that its schema should always be inlined
    // - The generator settings specify that all schemas should be inlined
    // - The inner type is a unit struct, which would cause an unsatisfiable schema due to mismatched `type`.
    //    In this case, we replace its type with "object" in `apply_internal_enum_variant_tag`
    // - The inner schema specified `"additionalProperties": false` or `"unevaluatedProperties": false`,
    //    since that would disallow the variant tag. If additional/unevaluatedProperties is in the top-level
    //    schema, then we can leave it there, because it will "see" the variant tag property. But if it is
    //    nested e.g. in an `allOf`, then it must be removed, which is why we run `AllowUnknownProperties`
    //    but only on immediate subschemas.

    let mut transform = AllowUnknownProperties::default();
    transform_immediate_subschemas(&mut transform, &mut schema);

    if T::inline_schema()
        || generator.settings().inline_subschemas
        || schema.get("type").and_then(Value::as_str) == Some("null")
        || schema.get("additionalProperties").and_then(Value::as_bool) == Some(false)
        || schema.get("unevaluatedProperties").and_then(Value::as_bool) == Some(false)
        || transform.did_modify
    {
        return schema;
    }

    // ...otherwise, we can freely refer to the schema via a `$ref`
    generator.subschema_for::<T>()
}

// Helper for generating schemas for flattened enums and `Option` fields.
pub fn json_schema_for_flatten<T: ?Sized + JsonSchema>(
    generator: &mut SchemaGenerator,
    required: bool,
) -> Schema {
    /// Non-generic inner function to reduce monomorphization overhead
    fn inner(mut schema: Schema, is_optional: bool) -> Schema {
        // Special handling for externally-tagged enums with unit variants.
        // Unit variants are normally serialized as strings, but when flattened, are serialized
        // as objects like `{ "VariantName": null }`
        if let Some(unit_variants) = remove_unit_variants(&mut schema) {
            if let Value::Array(one_of) = schema
                .ensure_object()
                .entry("oneOf")
                .or_insert(Value::Array(Vec::new()))
            {
                one_of.extend(
                    unit_variants
                        .iter()
                        .filter_map(Value::as_str)
                        .map(|variant| {
                            json!({
                                "type": "object",
                                "properties": {
                                    variant: {
                                        "type": "null"
                                    }
                                },
                                "required": [variant],
                            })
                        }),
                );
            }
        }

        if is_optional {
            schema.remove("required");

            // Handle `Option<>` of externally/internally/adjacently-tagged enums
            if let Some(one_of) = schema.remove("oneOf") {
                // We can't just add `{}` to the existing `oneOf`, because its items must be
                // mutually-exclusive, and `{}` matches everything.
                flatten(
                    &mut schema,
                    json_schema!({
                        "anyOf": [
                            { "oneOf": one_of },
                            {}
                        ]
                    }),
                );
            }

            // Handle `Option<>` of untagged enums
            if let Some(Value::Array(any_of)) = schema.get_mut("anyOf") {
                let empty_object = Value::Object(Map::new());
                if !any_of.contains(&empty_object) {
                    any_of.push(empty_object);
                }
            }
        }

        // Always allow aditional/unevaluated properties, because the outer struct determines
        // whether it denies unknown fields.
        AllowUnknownProperties::default().transform(&mut schema);

        schema
    }

    fn remove_unit_variants(schema: &mut Schema) -> Option<Vec<Value>> {
        // For enums that only have unit variants, all variants are in `enum`
        if schema.get("type").and_then(Value::as_str) == Some("string") {
            // Remove both `"enum": [...]`...
            if let Some(Value::Array(a)) = schema.remove("enum") {
                // ...and `"type": "string"`, since the variants are not serialized as strings
                schema.remove("type");
                return Some(a);
            }
        }

        // For enums that have unit and other variants, unit variants are in the first `oneOf` item
        let one_of = schema.get_mut("oneOf")?.as_array_mut()?;
        let first = <&mut Schema>::try_from(one_of.get_mut(0)?).ok()?;
        if first.get("type").and_then(Value::as_str) == Some("string") {
            if let Some(Value::Array(a)) = first.remove("enum") {
                one_of.remove(0);
                return Some(a);
            }
        }

        None
    }

    inner(
        T::_schemars_private_non_optional_json_schema(generator),
        T::_schemars_private_is_option() && !required,
    )
}

#[derive(Default)]
struct AllowUnknownProperties {
    did_modify: bool,
}

impl Transform for AllowUnknownProperties {
    fn transform(&mut self, schema: &mut Schema) {
        if schema.get("additionalProperties").and_then(Value::as_bool) == Some(false) {
            schema.remove("additionalProperties");
            self.did_modify = true;
        }
        if schema.get("unevaluatedProperties").and_then(Value::as_bool) == Some(false) {
            schema.remove("unevaluatedProperties");
            self.did_modify = true;
        }

        transform_immediate_subschemas(self, schema);
    }
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
#[must_use]
pub fn new_unit_enum_variant(variant: &str) -> Schema {
    json_schema!({
        "type": "string",
        "const": variant,
    })
}

/// Hack to simulate specialization:
/// `<MaybeJsonSchemaWrapper<T>>::maybe_schema_id()` will resolve to either
/// - The inherent method `MaybeJsonSchemaWrapper::maybe_schema_id()` if T impls `JsonSchema`
///     - this returns `T::schema_id()`
/// - The trait method `NoJsonSchema::maybe_schema_id()` from the blanket impl otherwise
///     - this returns `core::any::type_name::<T>()``
#[doc(hidden)]
#[macro_export]
macro_rules! _schemars_maybe_schema_id {
    ($ty:ty) => {{
        #[allow(unused_imports)]
        use $crate::_private::{MaybeJsonSchemaWrapper, NoJsonSchema as _};

        <MaybeJsonSchemaWrapper<$ty>>::maybe_schema_id()
    }};
}

pub struct MaybeJsonSchemaWrapper<T: ?Sized>(core::marker::PhantomData<T>);

pub trait NoJsonSchema {
    #[must_use]
    fn maybe_schema_id() -> Cow<'static, str> {
        Cow::Borrowed(core::any::type_name::<Self>())
    }
}

impl<T: ?Sized> NoJsonSchema for T {}

impl<T: JsonSchema + ?Sized> MaybeJsonSchemaWrapper<T> {
    #[must_use]
    pub fn maybe_schema_id() -> Cow<'static, str> {
        T::schema_id()
    }
}

/// Create a schema for an externally tagged enum variant
#[allow(clippy::needless_pass_by_value)]
#[must_use]
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
    let is_unit = obj.get("type").and_then(Value::as_str) == Some("null");

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

pub fn insert_metadata_property_if_nonempty(
    schema: &mut Schema,
    key: &str,
    value: impl Into<String>,
) {
    let value: String = value.into();
    if !value.is_empty() {
        schema.insert(key.to_owned(), value.into());
    }
}

pub fn insert_validation_property(
    schema: &mut Schema,
    required_type: &str,
    key: &str,
    value: impl Into<Value>,
) {
    if schema.has_type(required_type) || (required_type == "number" && schema.has_type("integer")) {
        schema.insert(key.to_owned(), value.into());
    }
}

pub fn must_contain(schema: &mut Schema, substring: &str) {
    let escaped = regex_syntax::escape(substring);
    insert_validation_property(schema, "string", "pattern", escaped);
}

pub fn apply_inner_validation(schema: &mut Schema, f: fn(&mut Schema) -> ()) {
    if let Some(inner_schema) = schema.get_mut("items").and_then(|i| i.try_into().ok()) {
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
                }
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
    ["if", "allOf", "anyOf", "oneOf", "$ref"]
        .into_iter()
        .any(|k| schema_obj.contains_key(k))
}

pub(crate) fn allow_null(generator: &mut SchemaGenerator, schema: &mut Schema) {
    fn is_null_schema(value: &Value) -> bool {
        <&Schema>::try_from(value).is_ok_and(|s| s.has_type("null"))
    }

    match schema.try_as_object_mut() {
        Ok(obj) => {
            if obj.len() == 1
                && obj
                    .get("anyOf")
                    .and_then(Value::as_array)
                    .is_some_and(|a| a.iter().any(is_null_schema))
            {
                return;
            }

            if contains_immediate_subschema(obj) {
                *schema = json_schema!({
                    "anyOf": [
                        obj,
                        <()>::json_schema(generator)
                    ]
                });
                // No need to check `type`/`const`/`enum` because they're trivially not present
                return;
            }

            if let Some(instance_type) = obj.get_mut("type") {
                match instance_type {
                    Value::Array(array) => {
                        let null = Value::from("null");
                        if !array.contains(&null) {
                            array.push(null);
                        }
                    }
                    Value::String(string) => {
                        if string != "null" {
                            let current_type = core::mem::take(string).into();
                            *instance_type = Value::Array(vec![current_type, "null".into()]);
                        }
                    }
                    _ => {}
                }
            }

            if let Some(c) = obj.remove("const") {
                if !c.is_null() {
                    obj.insert("enum".to_string(), Value::Array(vec![c, Value::Null]));
                }
            } else if let Some(Value::Array(e)) = obj.get_mut("enum") {
                if !e.contains(&Value::Null) {
                    e.push(Value::Null);
                }
            }
        }
        Err(true) => {}
        Err(false) => {
            *schema = <()>::json_schema(generator);
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[test]
    fn nested_option_schemas() {
        let mut option_schema = schema_for!(Option<Result<i8, u8>>);
        option_schema.remove("title");
        let mut nested_option_schema = schema_for!(Option<Option<Option<Result<i8, u8>>>>);
        nested_option_schema.remove("title");

        assert_eq!(option_schema, nested_option_schema);
    }
}
