---
layout: default
title: Using Serde Attributes
parent: Examples
nav_order: 2
summary: 'Deriving JsonSchema on types that use #[serde] attributes to customise serialization behaviour.'
---

# Using Serde Attributes

One of the main aims of this library is compatibility with [Serde](https://github.com/serde-rs/serde). Any generated schema *should* match how [serde_json](https://github.com/serde-rs/json) would serialize/deserialize to/from JSON. To support this, Schemars will check for any `#[serde(...)]` attributes on types that derive `JsonSchema`, and adjust the generated schema accordingly.

The list of supported `#[serde]` attributes are [documented here]({{ site.baseurl }}{% link 1.1-attributes.md %}#supported-serde-attributes).

{% include example.md name="serde_attrs" %}
