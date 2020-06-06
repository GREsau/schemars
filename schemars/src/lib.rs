/*!
Generate JSON Schema documents from Rust code

## Basic Usage

If you don't really care about the specifics, the easiest way to generate a JSON schema for your types is to `#[derive(JsonSchema)]` and use the `schema_for!` macro. All fields of the type must also implement `JsonSchema` - Schemars implements this for many standard library types.

```rust
use schemars::{schema_for, JsonSchema};

#[derive(JsonSchema)]
pub struct MyStruct {
    pub my_int: i32,
    pub my_bool: bool,
    pub my_nullable_enum: Option<MyEnum>,
}

#[derive(JsonSchema)]
pub enum MyEnum {
    StringNewType(String),
    StructVariant { floats: Vec<f32> },
}

fn main() {
    let schema = schema_for!(MyStruct);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
```

<details>
<summary>Click to see the output JSON schema...</summary>

```json
{
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "MyStruct",
    "type": "object",
    "required": [
        "my_bool",
        "my_int"
    ],
    "properties": {
        "my_bool": {
            "type": "boolean"
        },
        "my_int": {
            "type": "integer",
            "format": "int32"
        },
        "my_nullable_enum": {
            "anyOf": [
                {
                    "$ref": "#/definitions/MyEnum"
                },
                {
                    "type": "null"
                }
            ]
        }
    },
    "definitions": {
        "MyEnum": {
            "anyOf": [
                {
                    "type": "object",
                    "required": [
                        "StringNewType"
                    ],
                    "properties": {
                        "StringNewType": {
                            "type": "string"
                        }
                    }
                },
                {
                    "type": "object",
                    "required": [
                        "StructVariant"
                    ],
                    "properties": {
                        "StructVariant": {
                            "type": "object",
                            "required": [
                                "floats"
                            ],
                            "properties": {
                                "floats": {
                                    "type": "array",
                                    "items": {
                                        "type": "number",
                                        "format": "float"
                                    }
                                }
                            }
                        }
                    }
                }
            ]
        }
    }
}
```
</details>

### Serde Compatibility

One of the main aims of this library is compatibility with [Serde](https://github.com/serde-rs/serde). Any generated schema *should* match how [serde_json](https://github.com/serde-rs/json) would serialize/deserialize to/from JSON. To support this, Schemars will check for any `#[serde(...)]` attributes on types that derive `JsonSchema`, and adjust the generated schema accordingly.

```rust
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MyStruct {
    #[serde(rename = "myNumber")]
    pub my_int: i32,
    pub my_bool: bool,
    #[serde(default)]
    pub my_nullable_enum: Option<MyEnum>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum MyEnum {
    StringNewType(String),
    StructVariant { floats: Vec<f32> },
}

fn main() {
    let schema = schema_for!(MyStruct);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
```

<details>
<summary>Click to see the output JSON schema...</summary>

```json
{
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "MyStruct",
    "type": "object",
    "required": [
        "myBool",
        "myNumber"
    ],
    "properties": {
        "myBool": {
            "type": "boolean"
        },
        "myNullableEnum": {
            "default": null,
            "anyOf": [
                {
                    "$ref": "#/definitions/MyEnum"
                },
                {
                    "type": "null"
                }
            ]
        },
        "myNumber": {
            "type": "integer",
            "format": "int32"
        }
    },
    "additionalProperties": false,
    "definitions": {
        "MyEnum": {
            "anyOf": [
                {
                    "type": "string"
                },
                {
                    "type": "object",
                    "required": [
                        "floats"
                    ],
                    "properties": {
                        "floats": {
                            "type": "array",
                            "items": {
                                "type": "number",
                                "format": "float"
                            }
                        }
                    }
                }
            ]
        }
    }
}
```
</details>

`#[serde(...)]` attributes can be overriden using `#[schemars(...)]` attributes, which behave identically (e.g. `#[schemars(rename_all = "camelCase")]`). You may find this useful if you want to change the generated schema without affecting Serde's behaviour, or if you're just not using Serde.

## Feature Flags
- `derive` (enabled by default) - provides `#[derive(JsonSchema)]` macro
- `impl_json_schema` - implements `JsonSchema` for Schemars types themselves

## Optional Dependencies
Schemars can implement `JsonSchema` on types from several popular crates, enabled via optional dependencies (dependency versions are shown in brackets):
- [`chrono`](https://crates.io/crates/chrono) (^0.4)
- [`indexmap`](https://crates.io/crates/indexmap) (^1.2)
- [`either`](https://crates.io/crates/either) (^1.3)
- [`uuid`](https://crates.io/crates/uuid) (^0.8)
- [`smallvec`](https://crates.io/crates/uuid) (^1.0)
- [`arrayvec`](https://crates.io/crates/arrayvec) (^0.5)
*/

/// The map type used by schemars types.
///
/// Currently a `BTreeMap` or `IndexMap` can be used, but this may change a different implementation
/// The `IndexMap` will be used when the `preserve_order` feature flag is set.
/// with a similar interface in a future version of schemars.
#[cfg(not(feature = "preserve_order"))]
pub type Map<K, V> = std::collections::BTreeMap<K, V>;
#[cfg(feature = "preserve_order")]
pub type Map<K, V> = indexmap::IndexMap<K, V>;
/// The set type used by schemars types.
///
/// Currently a `BTreeSet`, but this may change a different implementation
/// with a similar interface in a future version of schemars.
pub type Set<T> = std::collections::BTreeSet<T>;

/// A view into a single entry in a map, which may either be vacant or occupied.
/// This `enum` is constructed from the `entry` method on `BTreeMap` or `IndexMap` 
/// depending on the `preserve_order` feature flag.
#[cfg(not(feature = "preserve_order"))]
pub type MapEntry<'a, K, V> = std::collections::btree_map::Entry<'a, K, V>;
#[cfg(feature = "preserve_order")]
pub type MapEntry<'a, K, V> = indexmap::map::Entry<'a, K, V>;

mod flatten;
mod json_schema_impls;
#[macro_use]
mod macros;

/// JSON Schema generator and settings.
pub mod gen;
/// JSON Schema types.
pub mod schema;

#[cfg(feature = "schemars_derive")]
extern crate schemars_derive;
#[cfg(feature = "schemars_derive")]
pub use schemars_derive::*;

// Export serde_json so schemars_derive can use it
#[doc(hidden)]
pub use serde_json as _serde_json;

use schema::{Schema, SchemaObject};

/// A type which can be described as a JSON Schema document.
///
/// This is implemented for many Rust primitive and standard library types.
///
/// This can also be automatically derived on most custom types with `#[derive(JsonSchema)]`.
///
/// # Example
/// ```
/// use schemars::{schema_for, JsonSchema};
///
/// #[derive(JsonSchema)]
/// struct MyStruct {
///     foo: i32,
/// }
///
/// let my_schema = schema_for!(MyStruct);
/// ```
pub trait JsonSchema {
    /// Whether JSON Schemas generated for this type should be re-used where possible using the `$ref` keyword.
    ///
    /// For trivial types (such as primitives), this should return `false`. For more complex types, it should return `true`.
    /// For recursive types, this **must** return `true` to prevent infinite cycles when generating schemas.
    ///
    /// By default, this returns `true`.
    fn is_referenceable() -> bool {
        true
    }

    /// The name of the generated JSON Schema.
    ///
    /// This is used as the title for root schemas, and the key within the root's `definitions` property for subschemas.
    fn schema_name() -> String;

    /// Generates a JSON Schema for this type.
    ///
    /// If the returned schema depends on any [referenceable](JsonSchema::is_referenceable) schemas, then this method will
    /// add them to the [`SchemaGenerator`](gen::SchemaGenerator)'s schema definitions.
    ///
    /// This should not return a `$ref` schema.
    fn json_schema(gen: &mut gen::SchemaGenerator) -> Schema;

    /// Helper for generating schemas for flattened `Option` fields.
    ///
    /// This should not need to be called or implemented by code outside of `schemars`,
    /// and should not be considered part of the public API.
    #[doc(hidden)]
    fn json_schema_for_flatten(gen: &mut gen::SchemaGenerator) -> Schema {
        Self::json_schema(gen)
    }

    /// Helper for generating schemas for `Option` fields.
    ///
    /// This should not need to be called or implemented by code outside of `schemars`,
    /// and should not be considered part of the public API.
    #[doc(hidden)]
    fn add_schema_as_property(
        gen: &mut gen::SchemaGenerator,
        parent: &mut SchemaObject,
        name: String,
        metadata: Option<schema::Metadata>,
        required: bool,
    ) {
        let mut schema = gen.subschema_for::<Self>();
        schema = gen.apply_metadata(schema, metadata);

        let object = parent.object();
        if required {
            object.required.insert(name.clone());
        }
        object.properties.insert(name, schema);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn schema_object_for<T: JsonSchema>() -> schema::SchemaObject {
        schema_object(schema_for::<T>())
    }

    pub fn custom_schema_object_for<T: JsonSchema>(
        settings: gen::SchemaSettings,
    ) -> schema::SchemaObject {
        schema_object(custom_schema_for::<T>(settings))
    }

    pub fn schema_for<T: JsonSchema>() -> schema::Schema {
        custom_schema_for::<T>(Default::default())
    }

    pub fn custom_schema_for<T: JsonSchema>(settings: gen::SchemaSettings) -> schema::Schema {
        T::json_schema(&mut gen::SchemaGenerator::new(settings))
    }

    pub fn schema_object(schema: schema::Schema) -> schema::SchemaObject {
        match schema {
            schema::Schema::Object(o) => o,
            s => panic!("Schema was not an object: {:?}", s),
        }
    }
}
