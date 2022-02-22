use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use std::borrow::Cow;
use std::collections::*;

impl<T: JsonSchema> JsonSchema for [T] {
    no_ref_schema!();

    fn schema_id() -> Cow<'static, str> {
        Cow::Owned(format!("[{}]", T::schema_id()))
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

impl<T: JsonSchema> JsonSchema for BTreeSet<T> {
    no_ref_schema!();

    fn schema_id() -> Cow<'static, str> {
        Cow::Owned(format!("std::collections::BTreeSet<{}>", T::schema_id()))
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

forward_impl!((<T> JsonSchema for BinaryHeap<T>) => [T]);
forward_impl!((<T> JsonSchema for LinkedList<T>) => [T]);
forward_impl!((<T> JsonSchema for Vec<T>) => [T]);
forward_impl!((<T> JsonSchema for VecDeque<T>) => [T]);

forward_impl!((<T, H> JsonSchema for HashSet<T, H>) => BTreeSet<T>);
