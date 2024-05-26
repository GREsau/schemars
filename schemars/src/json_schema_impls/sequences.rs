use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use std::borrow::Cow;

macro_rules! seq_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            T: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> Cow<'static, str> {
                format!("Array_of_{}", T::schema_name()).into()
            }

            fn schema_id() -> Cow<'static, str> {
                format!("[{}]", T::schema_id()).into()
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                json_schema!({
                    "type": "array",
                    "items": gen.subschema_for::<T>(),
                })
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

            fn schema_name() -> Cow<'static, str> {
                format!("Set_of_{}", T::schema_name()).into()
            }

            fn schema_id() -> Cow<'static, str> {
                format!("Set<{}>", T::schema_id()).into()
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                json_schema!({
                    "type": "array",
                    "uniqueItems": true,
                    "items": gen.subschema_for::<T>(),
                })
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
