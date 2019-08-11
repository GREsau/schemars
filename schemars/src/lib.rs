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
