use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::{JsonSchema, Result};

macro_rules! seq_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            T: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> String {
                format!("Array_Of_{}", T::schema_name())
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Result {
                Ok(SchemaObject {
                    instance_type: Some(InstanceType::Array.into()),
                    array: ArrayValidation {
                        items: Some(gen.subschema_for::<T>()?.into()),
                        ..Default::default()
                    },
                    ..Default::default()
                }.into())
            }
        }
    };
}

seq_impl!(<T: Ord> JsonSchema for std::collections::BinaryHeap<T>);
seq_impl!(<T: Ord> JsonSchema for std::collections::BTreeSet<T>);
seq_impl!(<T: Eq + core::hash::Hash, H: core::hash::BuildHasher> JsonSchema for std::collections::HashSet<T, H>);
seq_impl!(<T> JsonSchema for std::collections::LinkedList<T>);
seq_impl!(<T> JsonSchema for Vec<T>);
seq_impl!(<T> JsonSchema for std::collections::VecDeque<T>);
