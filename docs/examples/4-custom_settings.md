---
layout: default
title: Custom Schema Settings
parent: Examples
nav_order: 4
summary: Generating a schema using custom settings which changes how Option<T> is handled.
---

# Custom Schema Settings

The `gen` module allows you to customise how schemas are generated. For example, the default behaviour for `Option<T>` is to include `null` in the schema's `type`s, but we can instead add a `nullable` property to its schema:

{% include example.md name="custom_settings" %}
