pub type Map<K, V> = std::collections::BTreeMap<K, V>;
pub type Set<T> = std::collections::BTreeSet<T>;

mod flatten;
mod json_schema_impls;
#[macro_use]
mod macros;

/// JSON Schema generator and settings.
pub mod gen;
/// JSON Schema types.
pub mod schema;

pub use schemars_derive::*;

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
    /// For recursive types, this *must* return `true` to prevent infinite cycles when generating schemas.
    ///
    /// By default, this returns `true`.
    fn is_referenceable() -> bool {
        true
    }

    /// The name of the generated JSON Schema.
    ///
    /// This is used as the title for root schemas, and the key within the `definitions` property for subschemas.
    fn schema_name() -> String;

    /// Generates a JSON Schema for this type.
    ///
    /// If the returned schema depends on any [referenceable](JsonSchema::is_referenceable) schemas, then this method will
    /// add them to the [`SchemaGenerator`](gen::SchemaGenerator)'s schema definitions.
    ///
    /// This should not return a `$ref` schema.
    fn json_schema(gen: &mut gen::SchemaGenerator) -> Schema;

    /// Helper for generating schemas for flattened `Option` fields.
    ///
    /// This should not need to be called or implemented by code outside of `schemars`.
    #[doc(hidden)]
    fn json_schema_optional(gen: &mut gen::SchemaGenerator) -> Schema {
        Self::json_schema(gen)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn schema_object_for<T: JsonSchema>() -> schema::SchemaObject {
        schema_object(schema_for::<T>())
    }

    pub fn custom_schema_object_for<T: JsonSchema>(
        settings: gen::SchemaSettings,
    ) -> schema::SchemaObject {
        schema_object(custom_schema_for::<T>(settings))
    }

    pub fn schema_for<T: JsonSchema>() -> schema::Schema {
        custom_schema_for::<T>(Default::default())
    }

    pub fn custom_schema_for<T: JsonSchema>(settings: gen::SchemaSettings) -> schema::Schema {
        T::json_schema(&mut gen::SchemaGenerator::new(settings))
    }

    pub fn schema_object(schema: schema::Schema) -> schema::SchemaObject {
        match schema {
            schema::Schema::Object(o) => o,
            s => panic!("Schema was not an object: {:?}", s),
        }
    }
}
