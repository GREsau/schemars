use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use indexmap::{IndexMap, IndexSet};
use std::collections::{HashMap, HashSet};

forward_impl!((<K: JsonSchema, V: JsonSchema, H> JsonSchema for IndexMap<K, V, H>) => HashMap<K, V, H>);
forward_impl!((<T: JsonSchema, H> JsonSchema for IndexSet<T, H>) => HashSet<T, H>);
