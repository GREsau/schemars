use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use im;

forward_impl!((<T: JsonSchema> JsonSchema for im::HashSet<T>) => std::collections::HashSet<T>);
forward_impl!((<K, V: JsonSchema> JsonSchema for im::HashMap<K, V>) => std::collections::HashMap<K, V>);
forward_impl!((<T: JsonSchema> JsonSchema for im::OrdSet<T>) => std::collections::HashSet<T>);
forward_impl!((<K, V: JsonSchema> JsonSchema for im::OrdMap<K, V>) => std::collections::HashMap<K, V>);
forward_impl!((<T: JsonSchema> JsonSchema for im::Vector<T>) => std::vec::Vec<T>);
