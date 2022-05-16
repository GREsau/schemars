---
layout: default
title: Feature Flags
nav_order: 5
permalink: /features/
---

# Feature Flags and Optional Dependencies
- `derive` (enabled by default) - provides `#[derive(JsonSchema)]` macro
- `impl_json_schema` - implements `JsonSchema` for Schemars types themselves
- `preserve_order` - keep the order of struct fields in `Schema` and `SchemaObject`

Schemars can implement `JsonSchema` on types from several popular crates, enabled via feature flags (dependency versions are shown in brackets):
- `chrono` - [chrono](https://crates.io/crates/chrono) (^0.4)
- `indexmap1` - [indexmap](https://crates.io/crates/indexmap) (^1.2)
- `either` - [either](https://crates.io/crates/either) (^1.3)
- `uuid08` - [uuid](https://crates.io/crates/uuid) (^0.8)
- `uuid1` - [uuid](https://crates.io/crates/uuid) (^1.0)
- `smallvec` - [smallvec](https://crates.io/crates/smallvec) (^1.0)
- `arrayvec05` - [arrayvec](https://crates.io/crates/arrayvec) (^0.5)
- `arrayvec07` - [arrayvec](https://crates.io/crates/arrayvec) (^0.7)
- `url` - [url](https://crates.io/crates/url) (^2.0)
- `bytes` - [bytes](https://crates.io/crates/bytes) (^1.0)
- `enumset` - [enumset](https://crates.io/crates/enumset) (^1.0)
- `rust_decimal` - [rust_decimal](https://crates.io/crates/rust_decimal) (^1.0)
- `bigdecimal` - [bigdecimal](https://crates.io/crates/bigdecimal) (^0.3)

For example, to implement `JsonSchema` on types from `chrono`, enable it as a feature in the `schemars` dependency in your `Cargo.toml` like so:

```toml
[dependencies]
schemars = { version = "0.8", features = ["chrono"] }
```
