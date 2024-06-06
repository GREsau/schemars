use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use vec1::Vec1;

impl<T> JsonSchema for Vec1<T>
where
    T: JsonSchema,
{
    no_ref_schema!();

    fn schema_name() -> String {
        format!("Array_at_least_size_1_of_{}", T::schema_name())
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
