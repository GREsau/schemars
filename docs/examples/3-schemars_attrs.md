---
title: Using Schemars Attributes
parent: Examples
nav_order: 3
summary: "Deriving JsonSchema on types that use #[schemars] attributes to customise serialization behaviour."
---

# Using Schemars Attributes

`#[serde(...)]` attributes can be overriden (or replaced) with `#[schemars(...)]` attributes, which behave identically. You may find this useful if you want to change the generated schema without affecting Serde's behaviour, or if you're just not using Serde.

{% include example.md name="schemars_attrs" %}
