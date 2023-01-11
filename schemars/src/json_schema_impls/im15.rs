use crate::JsonSchema;
use im15;

forward_impl!((<T: JsonSchema> JsonSchema for im15::HashSet<T>) => std::collections::HashSet<T>);
forward_impl!((<K, V: JsonSchema> JsonSchema for im15::HashMap<K, V>) => std::collections::HashMap<K, V>);
forward_impl!((<T: JsonSchema> JsonSchema for im15::OrdSet<T>) => std::collections::HashSet<T>);
forward_impl!((<K, V: JsonSchema> JsonSchema for im15::OrdMap<K, V>) => std::collections::HashMap<K, V>);
forward_impl!((<T: JsonSchema> JsonSchema for im15::Vector<T>) => std::vec::Vec<T>);
