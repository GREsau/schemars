use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::{JsonSchema, Map, Result};
use serde_json::json;

// Does not require T: JsonSchema.
impl<T> JsonSchema for [T; 0] {
    no_ref_schema!();

    fn schema_name() -> String {
        "Empty_Array".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Result {
        let mut extensions = Map::new();
        extensions.insert("maxItems".to_owned(), json!(0));
        Ok(SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            extensions,
            ..Default::default()
        }
        .into())
    }
}

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T: JsonSchema> JsonSchema for [T; $len] {
                no_ref_schema!();

                fn schema_name() -> String {
                    format!("Array_Size_{}_Of_{}", $len, T::schema_name())
                }

                fn json_schema(gen: &mut SchemaGenerator) -> Result {
                    let mut extensions = Map::new();
                    extensions.insert("minItems".to_owned(), json!($len));
                    extensions.insert("maxItems".to_owned(), json!($len));
                    Ok(SchemaObject {
                        instance_type: Some(InstanceType::Array.into()),
                        items: Some(gen.subschema_for::<T>()?.into()),
                        extensions,
                        ..Default::default()
                    }.into())
                }
            }
        )+
    }
}

array_impls! {
     1  2  3  4  5  6  7  8  9 10
    11 12 13 14 15 16 17 18 19 20
    21 22 23 24 25 26 27 28 29 30
    31 32
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
        assert_eq!(schema.extensions.get("minItems"), Some(&json!(8)));
        assert_eq!(schema.extensions.get("maxItems"), Some(&json!(8)));
        assert_eq!(schema.items, Some(SingleOrVec::from(schema_for::<i32>())));
    }
}
