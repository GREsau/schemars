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
* [`SchemaSettings`](https://docs.rs/schemars/latest/schemars/gen/struct.SchemaSettings.html), which defines what JSON Schema features should be used when generating schemas (for example, how `Option`s should be represented).
* [`SchemaGenerator`](https://docs.rs/schemars/latest/schemars/gen/struct.SchemaGenerator.html), which manages the generation of a schema document.

See the API documentation for more info on how to use those types for custom schema generation.

## Schema from Example Value

If you want a schema for a type that can't/doesn't implement `JsonSchema`, but does implement `serde::Serialize`, then you can generate a JSON schema from a value of that type using the [`schema_for_value!` macro](https://docs.rs/schemars/latest/schemars/macro.schema_for_value.html). However, this schema will generally be less precise than if the type implemented `JsonSchema` - particularly when it involves enums, since schemars will not make any assumptions about the structure of an enum based on a single variant.

```rust
let value = MyStruct { foo = 123 };
let my_schema = schema_for_value!(value);
```

<!-- TODO:
create and link to example
-->
