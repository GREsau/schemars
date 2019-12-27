---
layout: default
title: Overview
nav_order: 1
---

# Schemars

Schemars is a library to generate JSON Schema documents from Rust data structures.

This is built on Rust's trait system - any type which implements the [`JsonSchema`](https://docs.rs/schemars/latest/schemars/trait.JsonSchema.html) trait can have a JSON Schema generated describing that type. Schemars implements this on many standard library types, and provides a derive macro to automatically implement it on custom types.

One of the main aims of this library is compatibility with [Serde](https://github.com/serde-rs/serde). Any generated schema *should* match how [serde_json](https://github.com/serde-rs/json) would serialize/deserialize to/from JSON. To support this, Schemars will check for any `#[serde(...)]` attributes on types that derive `JsonSchema`, and adjust the generated schema accordingly.

## Basic Usage

If you don't really care about the specifics, the easiest way to generate a JSON schema for your types is to `#[derive(JsonSchema)]` and use the `schema_for!` macro. All fields of the type must also implement `JsonSchema` - Schemars implements this for many standard library types.

{% include example.md name="main" %}
