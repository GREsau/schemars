use crate::JsonSchema;
use indexmap2::{IndexMap, IndexSet};
use ordermap::{OrderMap, OrderSet};

forward_impl!((<K: JsonSchema, V: JsonSchema, H> JsonSchema for OrderMap<K, V, H>) => IndexMap<K, V>);
forward_impl!((<T: JsonSchema, H> JsonSchema for OrderSet<T, H>) => IndexSet<T>);
