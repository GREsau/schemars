---
layout: default
title: Implementing JsonSchema
nav_order: 3
permalink: /implementing/
---

# Implementing JsonSchema

[Deriving `JsonSchema`]({{ site.baseurl }}{% link 1-deriving.md %}) is usually the easiest way to enable JSON schema generation for your types. But if you need more customisation, you can also implement `JsonSchema` manually. This trait has two associated functions which must be implemented, and one which can optionally be implemented:

## schema_name
```rust
fn schema_name() -> String;
```

This function returns the name of the type's schema, which frequently is just the name of the type itself. The schema name is used as the title for root schemas, and the key within the root's `$defs` property for subschemas.

If two types return the same `schema_name`, then Schemars will consider them identical types. Because of this, if a type takes any generic type parameters, then its schema name should depend on the type arguments. For example, the imlementation of this function for `Vec<T> where T: JsonSchema` is:
```rust
fn schema_name() -> String {
    format!("Array_of_{}", T::schema_name())
}
```

`BTreeSet<T>`, `LinkedList<T>`, and similar collection types also use that implementation, since they produce identical JSON schemas so they can be considered the same type.

## json_schema
```rust
fn json_schema(gen: &mut gen::SchemaGenerator) -> Schema;
```

This function creates the JSON schema itself. The `gen` argument can be used to check the schema generation settings, or to get schemas for other types. If you do need schemas for other types, you should call the `gen.subschema_for::<T>()` method instead of `<T>::json_schema(gen)`, as `subschema_for` can add `T`'s schema to the root schema's `$defs` so that it does not need to be duplicated when used more than once.

`json_schema` should not return a `$ref` schema.

## is_referenceable (optional)
```rust
fn is_referenceable() -> bool;
```

If this function returns `true`, then Schemars can re-use the generate schema where possible by adding it to the root schema's `$defs` and having other schemas reference it using the `$ref` keyword. This can greatly simplify schemas that include a particular type multiple times, especially if that type's schema is fairly complex.

Generally, this should return `false` for types with simple schemas (such as primitives). For more complex types, it should return `true`. For recursive types, this **must** return `true` to prevent infinite cycles when generating schemas.

The default implementation of this function returns `true` to reduce the chance of someone inadvertently causing infinite cycles with recursive types.