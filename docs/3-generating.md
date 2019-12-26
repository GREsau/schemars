---
layout: default
title: Generating Schemas
nav_order: 4
permalink: /generating/
---

# Generating Schemas

The easiest way to generate a schema for a type that implements is to use the [`schema_for!` macro](https://docs.rs/schemars/latest/schemars/macro.schema_for.html), like so:
```rust
let my_schema = schema_for!(MyStruct);
```

This will create a schema that conforms to [JSON Schema Draft 7](https://json-schema.org/specification-links.html#draft-7), but this is liable to change in a future version of Schemars if support for other JSON Schema versions is added.

If you want more control over how the schema is generated, you can use the [`gen` module](https://docs.rs/schemars/latest/schemars/gen/). There are two main types in this module:
* [`SchemaSettings`](https://docs.rs/schemars/0.6.1/schemars/gen/struct.SchemaSettings.html), which defines what JSON Schema features should be used when generating schemas (for example, how `Option`s should be represented).
* [`SchemaGenerator`](https://docs.rs/schemars/0.6.1/schemars/gen/struct.SchemaGenerator.html), which manages the generation of a schema document.

See the API documentation for more info on how to use those types for custom schema generation.

<!-- TODO: 
create and link to example
-->
