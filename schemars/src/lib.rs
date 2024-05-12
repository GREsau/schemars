#![deny(unsafe_code)]
#![doc = include_str!("../README.md")]

mod flatten;
mod json_schema_impls;
mod schema;
mod ser;
#[macro_use]
mod macros;

/// This module is only public for use by `schemars_derive`. It should not need to be used by code
/// outside of `schemars`, and should not be considered part of the public API.
#[doc(hidden)]
pub mod _private;
pub mod gen;
pub mod visit;

#[cfg(feature = "schemars_derive")]
extern crate schemars_derive;
use std::borrow::Cow;

#[cfg(feature = "schemars_derive")]
pub use schemars_derive::*;

// Export serde_json so schemars_derive can use it
#[doc(hidden)]
pub use serde_json as _serde_json;

pub use schema::Schema;

/// A type which can be described as a JSON Schema document.
///
/// This is implemented for many Rust primitive and standard library types.
///
/// This can also be automatically derived on most custom types with `#[derive(JsonSchema)]`.
///
/// # Examples
/// Deriving an implementation:
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
///
/// When manually implementing `JsonSchema`, as well as determining an appropriate schema,
/// you will need to determine an appropriate name and ID for the type.
/// For non-generic types, the type name/path are suitable for this:
/// ```
/// use schemars::{gen::SchemaGenerator, Schema, JsonSchema};
/// use std::borrow::Cow;
///
/// struct NonGenericType;
///
/// impl JsonSchema for NonGenericType {
///     fn schema_name() -> String {
///         // Exclude the module path to make the name in generated schemas clearer.
///         "NonGenericType".to_owned()
///     }
///
///     fn schema_id() -> Cow<'static, str> {
///         // Include the module, in case a type with the same name is in another module/crate
///         Cow::Borrowed(concat!(module_path!(), "::NonGenericType"))
///     }
///
///     fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
///         todo!()
///     }
/// }
///
/// assert_eq!(NonGenericType::schema_id(), <&mut NonGenericType>::schema_id());
/// ```
///
/// But generic type parameters which may affect the generated schema should typically be included in the name/ID:
/// ```
/// use schemars::{gen::SchemaGenerator, Schema, JsonSchema};
/// use std::{borrow::Cow, marker::PhantomData};
///
/// struct GenericType<T>(PhantomData<T>);
///
/// impl<T: JsonSchema> JsonSchema for GenericType<T> {
///     fn schema_name() -> String {
///         format!("GenericType_{}", T::schema_name())
///     }
///
///     fn schema_id() -> Cow<'static, str> {
///         Cow::Owned(format!(
///             "{}::GenericType<{}>",
///             module_path!(),
///             T::schema_id()
///         ))
///     }
///
///     fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
///         todo!()
///     }
/// }
///
/// assert_eq!(<GenericType<i32>>::schema_id(), <&mut GenericType<&i32>>::schema_id());
/// ```
///

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

    /// Returns a string that uniquely identifies the schema produced by this type.
    ///
    /// This does not have to be a human-readable string, and the value will not itself be included in generated schemas.
    /// If two types produce different schemas, then they **must** have different `schema_id()`s,
    /// but two types that produce identical schemas should *ideally* have the same `schema_id()`.
    ///
    /// The default implementation returns the same value as `schema_name()`.
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
