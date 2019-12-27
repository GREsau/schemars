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

<div class="indented">

### impl_json_schema
Implements `JsonSchema` on Schemars types themselves.

### chrono
Implements `JsonSchema` on all [Chrono](https://github.com/chronotope/chrono) types which are serializable by Serde.

### indexmap
Implements `JsonSchema` on `IndexMap` and `IndexSet` from [indexmap](https://github.com/bluss/indexmap).

### either
Implements `JsonSchema` on [`Either`](https://github.com/bluss/either).

### uuid
Implements `JsonSchema` on [`Uuid`](https://github.com/uuid-rs/uuid).

</div>
