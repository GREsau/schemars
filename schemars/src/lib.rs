#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

/// The map type used by schemars types.
///
/// Currently a `BTreeMap` or `IndexMap` can be used, but this may change to a different implementation
/// with a similar interface in a future version of schemars.
/// The `IndexMap` will be used when the `preserve_order` feature flag is set.
#[cfg(not(feature = "preserve_order"))]
pub type Map<K, V> = std::collections::BTreeMap<K, V>;
#[cfg(feature = "preserve_order")]
pub type Map<K, V> = indexmap::IndexMap<K, V>;
/// The set type used by schemars types.
///
/// Currently a `BTreeSet`, but this may change to a different implementation
/// with a similar interface in a future version of schemars.
pub type Set<T> = std::collections::BTreeSet<T>;

/// A view into a single entry in a map, which may either be vacant or occupied.
//
/// This is constructed from the `entry` method on `BTreeMap` or `IndexMap`,
/// depending on whether the `preserve_order` feature flag is set.
#[cfg(not(feature = "preserve_order"))]
pub type MapEntry<'a, K, V> = std::collections::btree_map::Entry<'a, K, V>;
#[cfg(feature = "preserve_order")]
pub type MapEntry<'a, K, V> = indexmap::map::Entry<'a, K, V>;

mod flatten;
mod json_schema_impls;
mod ser;
#[macro_use]
mod macros;

/// This module is only public for use by `schemars_derive`. It should not need to be used by code
/// outside of `schemars`, and should not be considered part of the public API.
#[doc(hidden)]
pub mod _private;
pub mod gen;
pub mod schema;
pub mod visit;

#[cfg(feature = "schemars_derive")]
extern crate schemars_derive;
use std::borrow::Cow;

#[cfg(feature = "schemars_derive")]
pub use schemars_derive::*;

// Export serde_json so schemars_derive can use it
#[doc(hidden)]
pub use serde_json as _serde_json;

use schema::Schema;

/// A type which can be described as a JSON Schema document.
///
/// This is implemented for many Rust primitive and standard library types.
///
/// This can also be automatically derived on most custom types with `#[derive(JsonSchema)]`.
///
/// # Example
/// ```
/// use schemars::{schema_for, JsonSchema};
///
/// #[derive(JsonSchema)]
/// struct MyStruct {
///     foo: i32,
/// }
///
/// let my_schema = schema_for!(MyStruct);
/// ```
pub trait JsonSchema {
    /// Whether JSON Schemas generated for this type should be re-used where possible using the `$ref` keyword.
    ///
    /// For trivial types (such as primitives), this should return `false`. For more complex types, it should return `true`.
    /// For recursive types, this **must** return `true` to prevent infinite cycles when generating schemas.
    ///
    /// By default, this returns `true`.
    fn is_referenceable() -> bool {
        true
    }

    /// The name of the generated JSON Schema.
    ///
    /// This is used as the title for root schemas, and the key within the root's `definitions` property for subschemas.
    fn schema_name() -> String;

    // TODO document
    fn schema_id() -> Cow<'static, str> {
        Cow::Owned(Self::schema_name())
    }

    /// Generates a JSON Schema for this type.
    ///
    /// If the returned schema depends on any [referenceable](JsonSchema::is_referenceable) schemas, then this method will
    /// add them to the [`SchemaGenerator`](gen::SchemaGenerator)'s schema definitions.
    ///
    /// This should not return a `$ref` schema.
    fn json_schema(gen: &mut gen::SchemaGenerator) -> Schema;

    // TODO document and bring into public API?
    #[doc(hidden)]
    fn _schemars_private_non_optional_json_schema(gen: &mut gen::SchemaGenerator) -> Schema {
        Self::json_schema(gen)
    }

    // TODO document and bring into public API?
    #[doc(hidden)]
    fn _schemars_private_is_option() -> bool {
        false
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn schema_object_for<T: JsonSchema>() -> schema::SchemaObject {
        schema_object(schema_for::<T>())
    }

    pub fn schema_for<T: JsonSchema>() -> schema::Schema {
        let mut gen = gen::SchemaGenerator::default();
        T::json_schema(&mut gen)
    }

    pub fn schema_object(schema: schema::Schema) -> schema::SchemaObject {
        match schema {
            schema::Schema::Object(o) => o,
            s => panic!("Schema was not an object: {:?}", s),
        }
    }
}
