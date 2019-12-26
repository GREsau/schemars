---
layout: default
title: Deriving JsonSchema
parent: Examples
nav_order: 1
summary: Deriving JsonSchema on a struct and enum.
---

# Deriving JsonSchema

This is the simplest usage of Schemars. Both types are made to derive `JsonSchema`, and the `schema_for!` macro is used to generate the schema itself.

{% include example.md name="main" %}
