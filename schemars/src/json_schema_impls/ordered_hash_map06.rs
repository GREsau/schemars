use crate::JsonSchema;
use alloc::collections::{BTreeMap, BTreeSet};
use ordered_hash_map06::{OrderedHashMap, OrderedHashSet};

forward_impl!((<K: JsonSchema, V: JsonSchema, S> JsonSchema for OrderedHashMap<K, V, S>) => BTreeMap<K, V>);
forward_impl!((<T: JsonSchema, S> JsonSchema for OrderedHashSet<T, S>) => BTreeSet<T>);
