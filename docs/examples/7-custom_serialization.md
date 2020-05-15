---
layout: default
title: Custom Serialization
parent: Examples
nav_order: 7
summary: >-
  If a field has a #[serde(with = "path")] attribute where "path" is not a type that implements JsonSchema,
  then in order to derive JsonSchema on the type, it must also have a #[schemars(with = "Type")] attribute,
  where "Type" implements JsonSchema.
---

# Deriving JsonSchema with Fields Using Custom Serialization

Serde allows you to change how a field is (de)serialized by setting a [`#[serde(with = "path")]`](https://serde.rs/field-attrs.html#with) attribute, where `$path::serialize` and `$path::deserialize` must be functions with the correct signature. Schemars supports the same attribute, but `path` must be a type implementing `JsonSchema`.

In order to derive `JsonSchema` on a type which includes a `#[serde(with = "path")]` attribute where `path` is not  a type implementing `JsonSchema`, you'll need to override it with a suitable `#[schemars(with = "Type")]` or `#[schemars(schema_with = "path")]` attribute.

{% include example.md name="custom_serialization" %}

Note that the `default` values in the schema are serialized as strings where appropriate.
