use crate::gen::SchemaGenerator;
use crate::{JsonSchema, Schema};
use std::borrow::Cow;
use std::num::*;

macro_rules! nonzero_unsigned_impl {
    ($type:ty => $primitive:ty) => {
        impl JsonSchema for $type {
            always_inline!();

            fn schema_name() -> Cow<'static, str> {
                stringify!($type).into()
            }

            fn schema_id() -> Cow<'static, str> {
                stringify!(std::num::$type).into()
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                let mut schema = <$primitive>::json_schema(gen);
                let object = schema.ensure_object();
                object.insert("not".to_owned(), serde_json::json!({
                    "const": 0
                }));
                schema
            }
        }
    };
}

nonzero_unsigned_impl!(NonZeroI8 => i8);
nonzero_unsigned_impl!(NonZeroI16 => i16);
nonzero_unsigned_impl!(NonZeroI32 => i32);
nonzero_unsigned_impl!(NonZeroI64 => i64);
nonzero_unsigned_impl!(NonZeroI128 => i128);
nonzero_unsigned_impl!(NonZeroIsize => isize);
