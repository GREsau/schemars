---
layout: default
title: Serialize Enum as Number (serde_repr)
parent: Examples
nav_order: 8
summary: >-
  Generating a schema for with a C-like enum compatible with serde_repr.
---

# Serialize Enum as Number (serde_repr Compatibility)

If you use the `#[repr(...)]` attribute on an enum to give it a C-like representation, then you may also want to use the [serde_repr](https://github.com/dtolnay/serde-repr) crate to serialize the enum values as numbers. In this case, you should use the corresponding `JsonSchema_repr` derive to ensure the schema for your type reflects how serde formats your type.

{% include example.md name="enum_repr" %}
