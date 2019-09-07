#![allow(clippy::large_enum_variant)]

pub type Map<K, V> = std::collections::BTreeMap<K, V>;
pub type Set<T> = std::collections::BTreeSet<T>;

mod error;
mod json_schema_impls;
#[macro_use]
mod macros;

pub mod gen;
pub mod schema;

pub use error::*;
pub use schemars_derive::*;

pub trait JsonSchema {
    fn is_referenceable() -> bool {
        true
    }

    fn schema_name() -> String;

    fn json_schema(gen: &mut gen::SchemaGenerator) -> Result;
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn schema_for<T: JsonSchema>() -> schema::Schema {
        match T::json_schema(&mut gen::SchemaGenerator::default()) {
            Ok(s) => s,
            Err(e) => panic!(
                "Couldn't generate schema object for {}: {}",
                T::schema_name(),
                e
            ),
        }
    }

    pub fn schema_object_for<T: JsonSchema>() -> schema::SchemaObject {
        match schema_for::<T>() {
            schema::Schema::Object(o) => o,
            s => panic!("Schema for {} was not an object: {:?}", T::schema_name(), s),
        }
    }
}
