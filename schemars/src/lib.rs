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
        match T::json_schema(&mut gen::SchemaGenerator::new(settings)) {
            Ok(s) => s,
            Err(e) => panic!(
                "Couldn't generate schema object for {}: {}",
                T::schema_name(),
                e
            ),
        }
    }

    pub fn schema_object(schema: schema::Schema) -> schema::SchemaObject {
        match schema {
            schema::Schema::Object(o) => o,
            s => panic!("Schema was not an object: {:?}", s),
        }
    }
}
