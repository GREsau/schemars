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
        if let Some(obj) = schema.as_object_mut() {
            obj.insert("my_property".to_string(), serde_json::json!("hello world"));
        }

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

The same example with a `fn` transform`:
```
# use schemars::{Schema, json_schema};
use schemars::transform::transform_subschemas;

fn add_property(schema: &mut Schema) {
    if let Some(obj) = schema.as_object_mut() {
        obj.insert("my_property".to_string(), serde_json::json!("hello world"));
    }

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
    if let Some(obj) = schema.as_object_mut() {
        obj.insert("my_property".to_string(), serde_json::json!("hello world"));
    }
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
use serde_json::{json, Value};

/// Trait used to modify a constructed schema and optionally its subschemas.
///
/// See the [module documentation](self) for more details on implementing this trait.
pub trait Transform {
    /// Applies the transform to the given [`Schema`].
    ///
    /// When overriding this method, you may want to call the [`transform_subschemas`] function to also transform any subschemas.
    fn transform(&mut self, schema: &mut Schema);

    // Not public API
    // Hack to enable implementing Debug on Box<dyn GenTransform> even though closures don't implement Debug
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
        self(schema)
    }
}

/// Applies the given [`Transform`] to all direct subschemas of the [`Schema`].
pub fn transform_subschemas<T: Transform + ?Sized>(t: &mut T, schema: &mut Schema) {
    if let Some(obj) = schema.as_object_mut() {
        for (key, value) in obj {
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
                        t.transform(subschema)
                    }
                }
                "allOf" | "anyOf" | "oneOf" | "prefixItems" => {
                    if let Some(array) = value.as_array_mut() {
                        for value in array {
                            if let Ok(subschema) = value.try_into() {
                                t.transform(subschema)
                            }
                        }
                    }
                }
                // Support `items` array even though this is not allowed in draft 2020-12 (see above comment)
                "items" => {
                    if let Some(array) = value.as_array_mut() {
                        for value in array {
                            if let Ok(subschema) = value.try_into() {
                                t.transform(subschema)
                            }
                        }
                    } else if let Ok(subschema) = value.try_into() {
                        t.transform(subschema)
                    }
                }
                "properties" | "patternProperties" | "$defs" | "definitions" => {
                    if let Some(obj) = value.as_object_mut() {
                        for value in obj.values_mut() {
                            if let Ok(subschema) = value.try_into() {
                                t.transform(subschema)
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// A helper struct that can wrap a non-recursive [`Transform`] (i.e. one that does not apply to subschemas) into a recursive one.
///
/// Its implementation of `Transform` will first apply the inner transform to the "parent" schema, and then its subschemas (and their subschemas, and so on).
///
/// # Example
/// ```
/// # use schemars::{Schema, json_schema};
/// use schemars::transform::{Transform, RecursiveTransform};
///
/// let mut transform = RecursiveTransform(|schema: &mut Schema| {
///     if let Some(obj) = schema.as_object_mut() {
///         obj.insert("my_property".to_string(), serde_json::json!("hello world"));
///     }
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
/// This is useful for dialects of JSON Schema (e.g. OpenAPI 3.0) that do not support booleans as schemas.
#[derive(Debug, Clone)]
pub struct ReplaceBoolSchemas {
    /// When set to `true`, a schema's `additionalProperties` property will not be changed from a boolean.
    pub skip_additional_properties: bool,
}

impl Transform for ReplaceBoolSchemas {
    fn transform(&mut self, schema: &mut Schema) {
        if let Some(obj) = schema.as_object_mut() {
            if self.skip_additional_properties {
                if let Some((ap_key, ap_value)) = obj.remove_entry("additionalProperties") {
                    transform_subschemas(self, schema);

                    if let Some(obj) = schema.as_object_mut() {
                        obj.insert(ap_key, ap_value);
                    }

                    return;
                }
            }

            transform_subschemas(self, schema);
        } else {
            schema.ensure_object();
        }
    }
}

/// Restructures JSON Schema objects so that the `$ref` property will never appear alongside any other properties.
/// This also applies to subschemas.
///
/// This is useful for versions of JSON Schema (e.g. Draft 7) that do not support other properties alongside `$ref`.
#[derive(Debug, Clone)]
pub struct RemoveRefSiblings;

impl Transform for RemoveRefSiblings {
    fn transform(&mut self, schema: &mut Schema) {
        transform_subschemas(self, schema);

        if let Some(obj) = schema.as_object_mut() {
            if obj.len() > 1 {
                if let Some(ref_value) = obj.remove("$ref") {
                    if let Value::Array(all_of) =
                        obj.entry("allOf").or_insert(Value::Array(Vec::new()))
                    {
                        all_of.push(json!({
                            "$ref": ref_value
                        }));
                    }
                }
            }
        }
    }
}

/// Removes the `examples` schema property and (if present) set its first value as the `example` property.
/// This also applies to subschemas.
///
/// This is useful for dialects of JSON Schema (e.g. OpenAPI 3.0) that do not support the `examples` property.
#[derive(Debug, Clone)]
pub struct SetSingleExample;

impl Transform for SetSingleExample {
    fn transform(&mut self, schema: &mut Schema) {
        transform_subschemas(self, schema);

        if let Some(obj) = schema.as_object_mut() {
            if let Some(Value::Array(examples)) = obj.remove("examples") {
                if let Some(first_example) = examples.into_iter().next() {
                    obj.insert("example".into(), first_example);
                }
            }
        }
    }
}

/// Replaces the `const` schema property with a single-valued `enum` property.
/// This also applies to subschemas.
///
/// This is useful for dialects of JSON Schema (e.g. OpenAPI 3.0) that do not support the `const` property.
#[derive(Debug, Clone)]
pub struct ReplaceConstValue;

impl Transform for ReplaceConstValue {
    fn transform(&mut self, schema: &mut Schema) {
        transform_subschemas(self, schema);

        if let Some(obj) = schema.as_object_mut() {
            if let Some(value) = obj.remove("const") {
                obj.insert("enum".into(), Value::Array(vec![value]));
            }
        }
    }
}

/// Rename the `prefixItems` schema property to `items`.
/// This also applies to subschemas.
///
/// If the schema contains both `prefixItems` and `items`, then this additionally renames `items` to `additionalItems`.
///
/// This is useful for versions of JSON Schema (e.g. Draft 7) that do not support the `prefixItems` property.
#[derive(Debug, Clone)]
pub struct ReplacePrefixItems;

impl Transform for ReplacePrefixItems {
    fn transform(&mut self, schema: &mut Schema) {
        transform_subschemas(self, schema);

        if let Some(obj) = schema.as_object_mut() {
            if let Some(prefix_items) = obj.remove("prefixItems") {
                let previous_items = obj.insert("items".to_owned(), prefix_items);

                if let Some(previous_items) = previous_items {
                    obj.insert("additionalItems".to_owned(), previous_items);
                }
            }
        }
    }
}
