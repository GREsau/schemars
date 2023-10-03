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

This function returns the human-readable friendly name of the type's schema, which frequently is just the name of the type itself. The schema name is used as the title for root schemas, and the key within the root's `definitions` property for subschemas.

NB in a future version of schemars, it's likely that this function will be changed to return a `Cow<'static, str>`.

## schema_id

```rust
fn schema_id() -> Cow<'static, str>;
```

This function returns a unique identifier of the type's schema - if two types return the same `schema_id`, then Schemars will consider them identical types. Because of this, if a type takes any generic type parameters, then its ID should depend on the type arguments. For example, the implementation of this function for `Vec<T> where T: JsonSchema` is:

```rust
fn schema_id() -> Cow<'static, str> {
    Cow::Owned(
        format!("[{}]", T::schema_id()))
}
```

`&mut Vec<&T>`, `LinkedList<T>`, `Mutex<LinkedList<Arc<T>>>`, and similar collection types also use that implementation, since they produce identical JSON schemas so they can be considered the same type.

For a type with no generic type arguments, a reasonable implementation of this function would be to return the type name including module path (in case there is a type with the same name in another module/crate), e.g.:

```rust
impl JsonSchema for NonGenericType {
    fn schema_name() -> String {
        // Exclude the module path to make the name in generated schemas clearer.
        "NonGenericType".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        // Include the module, in case a type with the same name is in another module/crate
        Cow::Borrowed(concat!(module_path!(), "::NonGenericType"))
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        todo!()
    }
}
```

## json_schema

```rust
fn json_schema(gen: &mut gen::SchemaGenerator) -> Schema;
```

This function creates the JSON schema itself. The `gen` argument can be used to check the schema generation settings, or to get schemas for other types. If you do need schemas for other types, you should call the `gen.subschema_for::<T>()` method instead of `<T>::json_schema(gen)`, as `subschema_for` can add `T`'s schema to the root schema's `definitions` so that it does not need to be duplicated when used more than once.

`json_schema` should not return a `$ref` schema.

## is_referenceable (optional)

```rust
fn is_referenceable() -> bool;
```

If this function returns `true`, then Schemars can re-use the generate schema where possible by adding it to the root schema's `definitions` and having other schemas reference it using the `$ref` keyword. This can greatly simplify schemas that include a particular type multiple times, especially if that type's schema is fairly complex.

Generally, this should return `false` for types with simple schemas (such as primitives). For more complex types, it should return `true`. For recursive types, this **must** return `true` to prevent infinite cycles when generating schemas.

The default implementation of this function returns `true` to reduce the chance of someone inadvertently causing infinite cycles with recursive types.
