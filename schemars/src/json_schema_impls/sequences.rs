use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;

macro_rules! seq_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            T: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> String {
                format!("Array_of_{}", T::schema_name())
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                SchemaObject {
                    instance_type: Some(InstanceType::Array.into()),
                    array: Some(Box::new(ArrayValidation {
                        items: Some(gen.subschema_for::<T>().into()),
                        ..Default::default()
                    })),
                    ..Default::default()
                }
                .into()
            }
        }
    };
}

macro_rules! set_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            T: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> String {
                format!("Set_of_{}", T::schema_name())
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                SchemaObject {
                    instance_type: Some(InstanceType::Array.into()),
                    array: Some(Box::new(ArrayValidation {
                        unique_items: Some(true),
                        items: Some(gen.subschema_for::<T>().into()),
                        ..Default::default()
                    })),
                    ..Default::default()
                }
                .into()
            }
        }
    };
}

seq_impl!(<T> JsonSchema for std::collections::BinaryHeap<T>);
seq_impl!(<T> JsonSchema for std::collections::LinkedList<T>);
seq_impl!(<T> JsonSchema for [T]);
seq_impl!(<T> JsonSchema for Vec<T>);
seq_impl!(<T> JsonSchema for std::collections::VecDeque<T>);

set_impl!(<T> JsonSchema for std::collections::BTreeSet<T>);
set_impl!(<T, H> JsonSchema for std::collections::HashSet<T, H>);
