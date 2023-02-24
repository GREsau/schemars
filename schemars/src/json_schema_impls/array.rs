use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;

pub struct EmptyArray<T>(pub [T; 0]);

// Does not require T: JsonSchema.
impl<T> JsonSchema for EmptyArray<T> {
    no_ref_schema!();

    fn schema_name() -> String {
        "EmptyArray".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                max_items: Some(0),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

impl<T: JsonSchema, const N: usize> JsonSchema for [T; N] {
    no_ref_schema!();

    fn schema_name() -> String {
        match N {
            0 => "EmptyArray".to_owned(),
            n => format!("Array_size_{}_of_{}", n, T::schema_name()),
        }
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                items: Some(gen.subschema_for::<T>().into()),
                max_items: Some(N as u32),
                min_items: Some(N as u32),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{schema_for, schema_object_for};
    use pretty_assertions::assert_eq;

    #[test]
    fn schema_for_array() {
        let schema = schema_object_for::<[i32; 8]>();
        assert_eq!(
            schema.instance_type,
            Some(SingleOrVec::from(InstanceType::Array))
        );
        let array_validation = schema.array.unwrap();
        assert_eq!(
            array_validation.items,
            Some(SingleOrVec::from(schema_for::<i32>()))
        );
        assert_eq!(array_validation.max_items, Some(8));
        assert_eq!(array_validation.min_items, Some(8));
    }

    // SomeStruct does not implement JsonSchema
    struct SomeStruct;

    #[test]
    fn schema_for_empty_array() {
        let schema = schema_object_for::<EmptyArray<SomeStruct>>();
        assert_eq!(
            schema.instance_type,
            Some(SingleOrVec::from(InstanceType::Array))
        );
        let array_validation = schema.array.unwrap();
        assert_eq!(array_validation.max_items, Some(0));
    }
}
