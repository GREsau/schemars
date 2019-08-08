pub type Map<K, V> = std::collections::BTreeMap<K, V>;
pub type Set<T> = std::collections::BTreeSet<T>;

pub mod gen;
pub mod schema;

mod error;
mod make_schema;
#[macro_use]
mod macros;

pub use error::*;
pub use make_schema::MakeSchema;

pub use schemars_derive::*;
