---
layout: default
title: Feature Flags
nav_order: 5
permalink: /features/
---

# Feature Flags and Optional Dependencies

Some functionality can be selectively enabled/disabled via [Cargo features](https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section). These can be enabled when you add Schemars to your crate's cargo.toml, e.g.
```toml
[dependencies]
schemars = { version = "0.6", features = ["chrono"] }
```

## Feature Flags
- `derive` (enabled by default) - provides `#[derive(JsonSchema)]` macro
- `impl_json_schema` - implements `JsonSchema` for Schemars types themselves
- `preserve_order` - keep the order of struct fields in `Schema` and `SchemaObject`

## Optional Dependencies
Schemars can implement `JsonSchema` on types from several popular crates, enabled via optional dependencies (dependency versions are shown in brackets):
- [`chrono`](https://crates.io/crates/chrono) (^0.4)
- [`indexmap`](https://crates.io/crates/indexmap) (^1.2)
- [`either`](https://crates.io/crates/either) (^1.3)
- [`uuid`](https://crates.io/crates/uuid) (^0.8)
- [`smallvec`](https://crates.io/crates/smallvec) (^1.0)
- [`arrayvec`](https://crates.io/crates/arrayvec) (^0.5)
- [`url`](https://crates.io/crates/url) (^2.0)
- [`bytes`](https://crates.io/crates/bytes) (^1.0)
