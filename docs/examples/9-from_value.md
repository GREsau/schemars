---
layout: default
title: Generate Schema from Example Value
parent: Examples
nav_order: 9
summary: >-
  Generating a schema for a serializable value.
---

# Generate Schema from Example Value

If you want a schema for a type that can't/doesn't implement `JsonSchema`, but does implement [`serde::Serialize`](https://docs.serde.rs/serde/trait.Serialize.html), then you can generate a JSON schema from a value of that type. However, this schema will generally be less precise than if the type implemented `JsonSchema` - particularly when it involves enums, since schemars will not make any assumptions about the structure of an enum based on a single variant.

{% include example.md name="from_value" %}

Note that the schema for the enum is not very useful in this case, since schemars doesn't know anything about the second variant.
