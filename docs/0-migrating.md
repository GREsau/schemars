---
title: Migrating from 0.8
nav_order: 2
has_children: true
has_toc: false
permalink: /migrating/
layout: default
---

# Migrating from 0.8 to 1.0 (or to 0.9)

<blockquote class="warning">
<p>Schemars 1.0 is still under development, and further changes may be introduced.
</blockquote>

## Optional dependencies

All optional dependencies are now suffixed by their version:

- `chrono` is now `chrono04`
- `either` is now `either1`
- `smallvec` is now `smallvec1`
- `url` is now `url2`
- `bytes` is now `bytes1`
- `rust_decimal` is now `rust_decimal1`
- `smol_str` is now `smol_str02` or `smol_str03`
- `semver` is now `semver1`
- `enumset`, `indexmap`, `uuid08`, `arrayvec05` and `bigdecimal03` have been removed
- `indexmap2`, `arrayvec07` and `bigdecimal04` are unchanged

## `Schema` is now a wrapper around `serde_json::Value`

`Schema` is now defined as a wrapper around a `serde_json::Value` (which must be a `Value::Bool` or `Value::Object`), rather than a struct with a field for each JSON schema keyword (with some intermediary types). `Schema` is now available as `schemars::Schema` instead of `schemars::schema::Schema`, and all other types that were in the `schemars::schema` module have now been removed. Functions that previously returned a `RootSchema` now just return a `Schema`.

A new macro `json_schema!(...)` is available to easily create new instances of `Schema`, which functions similarly to the [`serde_json::json!(...)` macro](https://docs.rs/serde_json/latest/serde_json/macro.json.html).

Here's how you might create and modify a `Schema` in schemars v0.8:

```rust
use schemars::schema::{InstanceType, ObjectValidation, Schema, SchemaObject};
use schemars::Map;

// Create a Schema for an object with property `foo`
let schema_object = SchemaObject {
    instance_type: Some(InstanceType::Object.into()),
    object: Some(Box::new(ObjectValidation {
        properties: Map::from_iter([("foo".to_owned(), true.into())]),
        ..Default::default()
    })),
    ..Default::default()
};
let schema: Schema = schema_object.into();

// Make the `foo` property required
let mut schema_object = schema.into_object();
let obj = schema_object.object();
obj.required.insert("foo".to_owned());
```

And the same thing in v1.0:

```rust
use schemars::{json_schema, Schema};

// Create a Schema for an object with property `foo`
let mut schema: Schema = json_schema!({
    "type": "object",
    "properties": {
        "foo": true
    }
});

// Make the `foo` property required
schema
    .ensure_object()
    .entry("required")
    .or_insert(serde_json::Value::Array(Vec::new()))
    .as_array_mut()
    .expect("`required` should be an array")
    .push("foo".into());
```

## `visit::Visitor` replaced with `transform::Transform`

The `visit` module and `Visitor` trait have been replace with `transform` and `Transform` respectively. Accordingly, these items have been renamed:

- `SchemaSettings::visitors` -> `SchemaSettings::transforms`
- `SchemaSettings::with_visitor` -> `SchemaSettings::with_transform`
- `SchemaGenerator::visitors_mut` -> `SchemaGenerator::transforms_mut`
- `GenVisitor` -> `GenTransform`
- `Visitor::visit_schema` -> `Transform::transform`
  - `visit_schema_object` and `visit_root_schema` methods have been removed
- `visit::visit_schema` -> `transform::transform_subschemas`
  - `visit_schema_object` and `visit_root_schema` functions have been removed

So if you had defined this `Visitor` in schemars 0.8:

```rust
use schemars::schema::SchemaObject;
use schemars::visit::{visit_schema_object, Visitor};

pub struct MyVisitor;

impl Visitor for MyVisitor {
    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        // First, make our change to this schema
        schema
            .extensions
            .insert("my_property".to_string(), serde_json::json!("hello world"));

        // Then delegate to default implementation to visit any subschemas
        visit_schema_object(self, schema);
    }
}

let mut schema = schemars::schema_for!(str);
MyVisitor.visit_root_schema(&mut schema);
```

Then the equivalent `Transform` in schemars 1.0 would be:

```rust
use schemars::transform::{transform_subschemas, Transform};
use schemars::Schema;

pub struct MyTransform;

impl Transform for MyTransform {
    fn transform(&mut self, schema: &mut Schema) {
        // First, make our change to this schema
        schema.insert("my_property".to_string(), serde_json::json!("hello world"));

        // Then apply the transform to any subschemas
        transform_subschemas(self, schema);
    }
}

let mut schema = schemars::schema_for!(str);
MyTransform.transform(&mut schema);
```

Also, since `Transform` is now implemented for functions that take a single `&mut Schema` argument, you could also define it as a function instead of a struct:

```rust
fn my_transform(schema: &mut Schema) {
    // First, make our change to this schema
    schema.insert("my_property".to_string(), serde_json::json!("hello world"));

    // Then apply the transform to any subschemas
    transform_subschemas(&mut my_transform, schema);
}

let mut schema = schemars::schema_for!(str);
my_transform(&mut schema);
// Or equivalently:
// my_transform.transform(&mut schema);
```

Finally, you can also use the `RecursiveTransform` newtype to convert a non-recursive `Transform` (i.e. one that does not transform subschemas) into a recursive one, like so:

```rust
fn my_transform2(schema: &mut Schema) {
    schema.insert("my_property".to_string(), serde_json::json!("hello world"));
}

let mut schema = schemars::schema_for!(str);
RecursiveTransform(my_transform2).transform(&mut schema);
```

## Changes to `#[validate(...)]` attributes

Since [adding support for `#[validate(...)]` attributes](https://graham.cool/schemars/v0/deriving/attributes/#supported-validator-attributes), the [Validator](https://github.com/Keats/validator) crate has made several changes to its supported attributes. Accordingly, Schemars 1.0 has updated its handling of `#[validate(...)]` attributes to match the latest version (currently 0.18.1) of the Validator crate - this removes some attributes, and changes the syntax of others:

- The `#[validate(phone)]`/`#[schemars(phone)]` attribute is removed. If you want the old behaviour of setting the "format" property on the generated schema, you can use `#[schemars(extend("format = "phone"))]` instead.
- The `#[validate(required_nested)]`/`#[schemars(required_nested)]` attribute is removed. If you want the old behaviour, you can use `#[schemars(required)]` instead.
- The `#[validate(regex = "...")]`/`#[schemars(regex = "...")]` attribute can no longer use `name = "value"` syntax. Instead, you can use:

  - `#[validate(regex(path = ...)]`
  - `#[schemars(regex(pattern = ...)]`
  - `#[schemars(pattern(...)]` (Garde-style)

- Similarly, the `#[validate(contains = "...")]`/`#[schemars(contains = "...")]` attribute can no longer use `name = "value"` syntax. Instead, you can use:

  - `#[validate(contains(pattern = ...))]`
  - `#[schemars(contains(pattern = ...))]`
  - `#[schemars(contains(...))]` (Garde-style)

As an alternative option, Schemars 1.0 also adds support for `#[garde(...)]` attributes used with the [Garde](https://github.com/jprochazk/garde) crate, along with equivalent `#[schemars(...)]` attributes. See [the documentation](https://graham.cool/schemars/deriving/attributes/#supported-validatorgarde-attributes) for a list of all supported attributes.
