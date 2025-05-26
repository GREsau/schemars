---
title: Feature Flags
nav_order: 6
permalink: /features/
---

# Feature Flags and Optional Dependencies

- `std` (enabled by default) - implements `JsonSchema` for types in the rust standard library (`JsonSchema` is still implemented on types in `core` and `alloc`, even when this feature is disabled). Disable this feature to use schemars in `no_std` environments.
- `derive` (enabled by default) - provides `#[derive(JsonSchema)]` macro
- `preserve_order` - keep the order of struct fields in `Schema` properties
- `raw_value` - implements `JsonSchema` for `serde_json::value::RawValue` (enables the serde_json `raw_value` feature)

Schemars can implement `JsonSchema` on types from several popular crates, enabled via feature flags (dependency versions are shown in brackets):

- `arrayvec07` - [arrayvec](https://crates.io/crates/arrayvec) (^0.7)
- `bigdecimal04` - [bigdecimal](https://crates.io/crates/bigdecimal) (^0.4)
- `bytes1` - [bytes](https://crates.io/crates/bytes) (^1.0)
- `chrono04` - [chrono](https://crates.io/crates/chrono) (^0.4)
- `either1` - [either](https://crates.io/crates/either) (^1.3)
- `indexmap2` - [indexmap](https://crates.io/crates/indexmap) (^2.0)
- `jiff02` - [jiff](https://crates.io/crates/jiff) (^0.2)
- `rust_decimal1` - [rust_decimal](https://crates.io/crates/rust_decimal) (^1.0)
- `semver1` - [semver](https://crates.io/crates/semver) (^1.0.9)
- `smallvec1` - [smallvec](https://crates.io/crates/smallvec) (^1.0)
- `smol_str02` - [smol_str](https://crates.io/crates/smol_str) (^0.2.1)
- `url2` - [url](https://crates.io/crates/url) (^2.0)
- `uuid1` - [uuid](https://crates.io/crates/uuid) (^1.0)

For example, to implement `JsonSchema` on types from `chrono`, enable it as a feature in the `schemars` dependency in your `Cargo.toml` like so:

```toml
[dependencies]
schemars = { version = "0.9.0", features = ["chrono04"] }
```
