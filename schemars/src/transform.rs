/*!
Contains the [`Transform`] trait, used to modify a constructed schema and optionally its subschemas.
This trait is automatically implemented for functions of the form `fn(&mut Schema) -> ()`.

# Recursive Transforms

To make a transform recursive (i.e. apply it to subschemas), you have two options:
1. call the [`transform_subschemas`] function within the transform function
2. wrap the `Transform` in a [`RecursiveTransform`]

# Examples

To add a custom property to all object schemas:

```
# use schemars::{Schema, json_schema};
use schemars::transform::{Transform, transform_subschemas};

pub struct MyTransform;

impl Transform for MyTransform {
    fn transform(&mut self, schema: &mut Schema) {
        // First, make our change to this schema
        schema.insert("my_property".to_string(), "hello world".into());

        // Then apply the transform to any subschemas
        transform_subschemas(self, schema);
    }
}

let mut schema = json_schema!({
    "type": "array",
    "items": {}
});

MyTransform.transform(&mut schema);

assert_eq!(
    schema,
    json_schema!({
        "type": "array",
        "items": {
            "my_property": "hello world"
        },
        "my_property": "hello world"
    })
);
```

The same example with a `fn` transform:
```
# use schemars::{Schema, json_schema};
use schemars::transform::transform_subschemas;

fn add_property(schema: &mut Schema) {
    schema.insert("my_property".to_string(), "hello world".into());

    transform_subschemas(&mut add_property, schema)
}

let mut schema = json_schema!({
    "type": "array",
    "items": {}
});

add_property(&mut schema);

assert_eq!(
    schema,
    json_schema!({
        "type": "array",
        "items": {
            "my_property": "hello world"
        },
        "my_property": "hello world"
    })
);
```

And the same example using a closure wrapped in a `RecursiveTransform`:
```
# use schemars::{Schema, json_schema};
use schemars::transform::{Transform, RecursiveTransform};

let mut transform = RecursiveTransform(|schema: &mut Schema| {
    schema.insert("my_property".to_string(), "hello world".into());
});

let mut schema = json_schema!({
    "type": "array",
    "items": {}
});

transform.transform(&mut schema);

assert_eq!(
    schema,
    json_schema!({
        "type": "array",
        "items": {
            "my_property": "hello world"
        },
        "my_property": "hello world"
    })
);
```

*/
use crate::Schema;
use crate::_alloc_prelude::*;
use alloc::collections::BTreeSet;
use serde_json::{json, Map, Value};

/// Trait used to modify a constructed schema and optionally its subschemas.
///
/// See the [module documentation](self) for more details on implementing this trait.
pub trait Transform {
    /// Applies the transform to the given [`Schema`].
    ///
    /// When overriding this method, you may want to call the [`transform_subschemas`] function to
    /// also transform any subschemas.
    fn transform(&mut self, schema: &mut Schema);

    // Not public API
    // Hack to enable implementing Debug on Box<dyn GenTransform> even though closures don't
    // implement Debug
    #[doc(hidden)]
    fn _debug_type_name(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(core::any::type_name::<Self>())
    }
}

impl<F> Transform for F
where
    F: FnMut(&mut Schema),
{
    fn transform(&mut self, schema: &mut Schema) {
        self(schema);
    }
}

/// Applies the given [`Transform`] to all direct subschemas of the [`Schema`].
pub fn transform_subschemas<T: Transform + ?Sized>(t: &mut T, schema: &mut Schema) {
    for (key, value) in schema.as_object_mut().into_iter().flatten() {
        // This is intentionally written to work with multiple JSON Schema versions, so that
        // users can add their own transforms on the end of e.g. `SchemaSettings::draft07()` and
        // they will still apply to all subschemas "as expected".
        // This is why this match statement contains both `additionalProperties` (which was
        // dropped in draft 2020-12) and `prefixItems` (which was added in draft 2020-12).
        match key.as_str() {
            "not"
            | "if"
            | "then"
            | "else"
            | "contains"
            | "additionalProperties"
            | "propertyNames"
            | "additionalItems" => {
                if let Ok(subschema) = value.try_into() {
                    t.transform(subschema);
                }
            }
            "allOf" | "anyOf" | "oneOf" | "prefixItems" => {
                if let Some(array) = value.as_array_mut() {
                    for value in array {
                        if let Ok(subschema) = value.try_into() {
                            t.transform(subschema);
                        }
                    }
                }
            }
            // Support `items` array even though this is not allowed in draft 2020-12 (see above
            // comment)
            "items" => {
                if let Some(array) = value.as_array_mut() {
                    for value in array {
                        if let Ok(subschema) = value.try_into() {
                            t.transform(subschema);
                        }
                    }
                } else if let Ok(subschema) = value.try_into() {
                    t.transform(subschema);
                }
            }
            "properties" | "patternProperties" | "$defs" | "definitions" => {
                if let Some(obj) = value.as_object_mut() {
                    for value in obj.values_mut() {
                        if let Ok(subschema) = value.try_into() {
                            t.transform(subschema);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// Similar to `transform_subschemas`, but only transforms subschemas that apply to the top-level
// object, e.g. "oneOf" but not "properties".
pub(crate) fn transform_immediate_subschemas<T: Transform + ?Sized>(
    t: &mut T,
    schema: &mut Schema,
) {
    for (key, value) in schema.as_object_mut().into_iter().flatten() {
        match key.as_str() {
            "if" | "then" | "else" => {
                if let Ok(subschema) = value.try_into() {
                    t.transform(subschema);
                }
            }
            "allOf" | "anyOf" | "oneOf" => {
                if let Some(array) = value.as_array_mut() {
                    for value in array {
                        if let Ok(subschema) = value.try_into() {
                            t.transform(subschema);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// A helper struct that can wrap a non-recursive [`Transform`] (i.e. one that does not apply to
/// subschemas) into a recursive one.
///
/// Its implementation of `Transform` will first apply the inner transform to the "parent" schema,
/// and then its subschemas (and their subschemas, and so on).
///
/// # Example
/// ```
/// # use schemars::{Schema, json_schema};
/// use schemars::transform::{Transform, RecursiveTransform};
///
/// let mut transform = RecursiveTransform(|schema: &mut Schema| {
///     schema.insert("my_property".to_string(), "hello world".into());
/// });
///
/// let mut schema = json_schema!({
///     "type": "array",
///     "items": {}
/// });
///
/// transform.transform(&mut schema);
///
/// assert_eq!(
///     schema,
///     json_schema!({
///         "type": "array",
///         "items": {
///             "my_property": "hello world"
///         },
///         "my_property": "hello world"
///     })
/// );
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_structs)]
pub struct RecursiveTransform<T>(pub T);

impl<T> Transform for RecursiveTransform<T>
where
    T: Transform,
{
    fn transform(&mut self, schema: &mut Schema) {
        self.0.transform(schema);
        transform_subschemas(self, schema);
    }
}

/// Replaces boolean JSON Schemas with equivalent object schemas.
/// This also applies to subschemas.
///
/// This is useful for dialects of JSON Schema (e.g. OpenAPI 3.0) that do not support booleans as
/// schemas.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ReplaceBoolSchemas {
    /// When set to `true`, a schema's `additionalProperties` property will not be changed from a
    /// boolean.
    pub skip_additional_properties: bool,
}

impl Transform for ReplaceBoolSchemas {
    fn transform(&mut self, schema: &mut Schema) {
        if let Some(obj) = schema.as_object_mut() {
            if self.skip_additional_properties {
                if let Some((ap_key, ap_value)) = obj.remove_entry("additionalProperties") {
                    transform_subschemas(self, schema);

                    schema.insert(ap_key, ap_value);

                    return;
                }
            }

            transform_subschemas(self, schema);
        } else {
            schema.ensure_object();
        }
    }
}

/// Restructures JSON Schema objects so that the `$ref` property will never appear alongside any
/// other properties. This also applies to subschemas.
///
/// This is useful for versions of JSON Schema (e.g. Draft 7) that do not support other properties
/// alongside `$ref`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct RemoveRefSiblings;

impl Transform for RemoveRefSiblings {
    fn transform(&mut self, schema: &mut Schema) {
        transform_subschemas(self, schema);

        if let Some(obj) = schema.as_object_mut().filter(|o| o.len() > 1) {
            if let Some(ref_value) = obj.remove("$ref") {
                if let Value::Array(all_of) = obj.entry("allOf").or_insert(Value::Array(Vec::new()))
                {
                    all_of.push(json!({
                        "$ref": ref_value
                    }));
                }
            }
        }
    }
}

/// Removes the `examples` schema property and (if present) set its first value as the `example`
/// property. This also applies to subschemas.
///
/// This is useful for dialects of JSON Schema (e.g. OpenAPI 3.0) that do not support the `examples`
/// property.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct SetSingleExample;

impl Transform for SetSingleExample {
    fn transform(&mut self, schema: &mut Schema) {
        transform_subschemas(self, schema);

        if let Some(Value::Array(examples)) = schema.remove("examples") {
            if let Some(first_example) = examples.into_iter().next() {
                schema.insert("example".into(), first_example);
            }
        }
    }
}

/// Replaces the `const` schema property with a single-valued `enum` property.
/// This also applies to subschemas.
///
/// This is useful for dialects of JSON Schema (e.g. OpenAPI 3.0) that do not support the `const`
/// property.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ReplaceConstValue;

impl Transform for ReplaceConstValue {
    fn transform(&mut self, schema: &mut Schema) {
        transform_subschemas(self, schema);

        if let Some(value) = schema.remove("const") {
            schema.insert("enum".into(), Value::Array(vec![value]));
        }
    }
}

/// Rename the `prefixItems` schema property to `items`.
/// This also applies to subschemas.
///
/// If the schema contains both `prefixItems` and `items`, then this additionally renames `items` to
/// `additionalItems`.
///
/// This is useful for versions of JSON Schema (e.g. Draft 7) that do not support the `prefixItems`
/// property.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ReplacePrefixItems;

impl Transform for ReplacePrefixItems {
    fn transform(&mut self, schema: &mut Schema) {
        transform_subschemas(self, schema);

        if let Some(prefix_items) = schema.remove("prefixItems") {
            let previous_items = schema.insert("items".to_owned(), prefix_items);

            if let Some(previous_items) = previous_items {
                schema.insert("additionalItems".to_owned(), previous_items);
            }
        }
    }
}

/// Adds a `"nullable": true` property to schemas that allow `null` types.
/// This also applies to subschemas.
///
/// This is useful for dialects of JSON Schema (e.g. OpenAPI 3.0) that use `nullable` instead of
/// explicit null types.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AddNullable {
    /// When set to `true`, `"null"` will also be removed from the schemas `type`.
    pub remove_null_type: bool,
    /// When set to `true`, a schema that has a type only allowing `null` will also have the
    /// equivalent `"const": null` inserted.
    pub add_const_null: bool,
}

impl Default for AddNullable {
    fn default() -> Self {
        Self {
            remove_null_type: true,
            add_const_null: true,
        }
    }
}

impl Transform for AddNullable {
    fn transform(&mut self, schema: &mut Schema) {
        if schema.has_type("null") {
            // has_type returned true so we know schema is an object
            let obj = schema.as_object_mut().unwrap();

            obj.insert("nullable".into(), true.into());

            // has_type returned true so we know "type" exists and is a string or array
            let ty = obj.get_mut("type").unwrap();
            let only_allows_null =
                ty.is_string() || ty.as_array().unwrap().iter().all(|v| v == "null");

            if only_allows_null {
                if self.add_const_null {
                    obj.insert("const".to_string(), Value::Null);

                    if self.remove_null_type {
                        obj.remove("type");
                    }
                } else if self.remove_null_type {
                    *ty = Value::Array(Vec::new());
                }
            } else if self.remove_null_type {
                // We know `type` is an array containing at least one non-null type
                let array = ty.as_array_mut().unwrap();
                array.retain(|t| t != "null");

                if array.len() == 1 {
                    *ty = array.remove(0);
                }
            }
        }

        transform_subschemas(self, schema);
    }
}

/// Replaces the `unevaluatedProperties` schema property with the `additionalProperties` property,
/// adding properties from a schema's subschemas to its `properties` where necessary.
/// This also applies to subschemas.
///
/// This is useful for versions of JSON Schema (e.g. Draft 7) that do not support the
/// `unevaluatedProperties` property.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ReplaceUnevaluatedProperties;

impl Transform for ReplaceUnevaluatedProperties {
    fn transform(&mut self, schema: &mut Schema) {
        transform_subschemas(self, schema);

        let Some(up) = schema.remove("unevaluatedProperties") else {
            return;
        };

        schema.insert("additionalProperties".to_owned(), up);

        let mut gather_property_names = GatherPropertyNames::default();
        gather_property_names.transform(schema);
        let property_names = gather_property_names.0;

        if property_names.is_empty() {
            return;
        }

        if let Some(properties) = schema
            .ensure_object()
            .entry("properties")
            .or_insert(Map::new().into())
            .as_object_mut()
        {
            for name in property_names {
                properties.entry(name).or_insert(true.into());
            }
        }
    }
}

// Helper for getting property names for all *immediate* subschemas
#[derive(Default)]
struct GatherPropertyNames(BTreeSet<String>);

impl Transform for GatherPropertyNames {
    fn transform(&mut self, schema: &mut Schema) {
        self.0.extend(
            schema
                .as_object()
                .iter()
                .filter_map(|o| o.get("properties"))
                .filter_map(Value::as_object)
                .flat_map(Map::keys)
                .cloned(),
        );

        transform_immediate_subschemas(self, schema);
    }
}
