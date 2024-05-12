use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use std::borrow::Cow;

// Does not require T: JsonSchema.
impl<T> JsonSchema for [T; 0] {
    no_ref_schema!();

    fn schema_name() -> String {
        "EmptyArray".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("[]")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "array",
            "maxItems": 0,
        })
    }
}

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T: JsonSchema> JsonSchema for [T; $len] {
                no_ref_schema!();

                fn schema_name() -> String {
                    format!("Array_size_{}_of_{}", $len, T::schema_name())
                }

                fn schema_id() -> Cow<'static, str> {
                    Cow::Owned(
                        format!("[{}; {}]", $len, T::schema_id()))
                }

                fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                    json_schema!({
                        "type": "array",
                        "items": serde_json::Value::from(gen.subschema_for::<T>()),
                        "minItems": $len,
                        "maxItems": $len,
                    })
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
