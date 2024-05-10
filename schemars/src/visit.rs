/*!
Contains the [`Visitor`] trait, used to recursively modify a constructed schema and its subschemas.

Sometimes you may want to apply a change to a schema, as well as all schemas contained within it.
The easiest way to achieve this is by defining a type that implements [`Visitor`].
All methods of `Visitor` have a default implementation that makes no change but recursively visits all subschemas.
When overriding one of these methods, you will *usually* want to still call this default implementation.

# Example
To add a custom property to all schemas:
```
use schemars::schema::SchemaObject;
use schemars::visit::{Visitor, visit_schema_object};

pub struct MyVisitor;

impl Visitor for MyVisitor {
    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        // First, make our change to this schema
        schema.extensions.insert("my_property".to_string(), serde_json::json!("hello world"));

        // Then delegate to default implementation to visit any subschemas
        visit_schema_object(self, schema);
    }
}
```
*/
use serde_json::{json, Value};

use crate::schema::Schema;

/// Trait used to recursively modify a constructed schema and its subschemas.
pub trait Visitor {
    /// Override this method to modify a [`Schema`] and (optionally) its subschemas.
    ///
    /// When overriding this method, you will usually want to call the [`visit_schema`] function to visit subschemas.
    fn visit_schema(&mut self, schema: &mut Schema) {
        visit_schema(self, schema)
    }
}

/// Visits all subschemas of the [`Schema`].
pub fn visit_schema<V: Visitor + ?Sized>(v: &mut V, schema: &mut Schema) {
    if let Some(obj) = schema.as_object_mut() {
        for (key, value) in obj {
            match key.as_str() {
                "not"
                | "if"
                | "then"
                | "else"
                | "additionalItems"
                | "contains"
                | "additionalProperties"
                | "propertyNames" => {
                    if let Ok(subschema) = value.try_into() {
                        v.visit_schema(subschema)
                    }
                }
                "allOf" | "anyOf" | "oneOf" => {
                    if let Some(array) = value.as_array_mut() {
                        for value in array {
                            if let Ok(subschema) = value.try_into() {
                                v.visit_schema(subschema)
                            }
                        }
                    }
                }
                "items" => {
                    if let Some(array) = value.as_array_mut() {
                        for value in array {
                            if let Ok(subschema) = value.try_into() {
                                v.visit_schema(subschema)
                            }
                        }
                    } else if let Ok(subschema) = value.try_into() {
                        v.visit_schema(subschema)
                    }
                }
                "properties" | "patternProperties" | "definitions" | "$defs" => {
                    if let Some(obj) = value.as_object_mut() {
                        for value in obj.values_mut() {
                            if let Ok(subschema) = value.try_into() {
                                v.visit_schema(subschema)
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// This visitor will replace all boolean JSON Schemas with equivalent object schemas.
///
/// This is useful for dialects of JSON Schema (e.g. OpenAPI 3.0) that do not support booleans as schemas.
#[derive(Debug, Clone)]
pub struct ReplaceBoolSchemas {
    /// When set to `true`, a schema's `additionalProperties` property will not be changed from a boolean.
    pub skip_additional_properties: bool,
}

impl Visitor for ReplaceBoolSchemas {
    fn visit_schema(&mut self, schema: &mut Schema) {
        if let Some(obj) = schema.as_object_mut() {
            if self.skip_additional_properties {
                if let Some((ap_key, ap_value)) = obj.remove_entry("additionalProperties") {
                    visit_schema(self, schema);

                    if let Some(obj) = schema.as_object_mut() {
                        obj.insert(ap_key, ap_value);
                    }

                    return;
                }
            }

            visit_schema(self, schema);
        } else {
            schema.ensure_object();
        }
    }
}

/// This visitor will restructure JSON Schema objects so that the `$ref` property will never appear alongside any other properties.
///
/// This is useful for dialects of JSON Schema (e.g. Draft 7) that do not support other properties alongside `$ref`.
#[derive(Debug, Clone)]
pub struct RemoveRefSiblings;

impl Visitor for RemoveRefSiblings {
    fn visit_schema(&mut self, schema: &mut Schema) {
        visit_schema(self, schema);

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

/// This visitor will remove the `examples` schema property and (if present) set its first value as the `example` property.
///
/// This is useful for dialects of JSON Schema (e.g. OpenAPI 3.0) that do not support the `examples` property.
#[derive(Debug, Clone)]
pub struct SetSingleExample;

impl Visitor for SetSingleExample {
    fn visit_schema(&mut self, schema: &mut Schema) {
        visit_schema(self, schema);

        if let Some(obj) = schema.as_object_mut() {
            if let Some(Value::Array(examples)) = obj.remove("examples") {
                if let Some(first_example) = examples.into_iter().next() {
                    obj.insert("example".into(), first_example);
                }
            }
        }
    }
}
