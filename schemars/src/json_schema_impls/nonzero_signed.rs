use crate::gen::SchemaGenerator;
use crate::{JsonSchema, Schema};
use std::borrow::Cow;
use std::num::*;

macro_rules! nonzero_unsigned_impl {
    ($type:ty => $primitive:ty) => {
        impl JsonSchema for $type {
            no_ref_schema!();

            fn schema_name() -> String {
                stringify!($type).to_owned()
            }

            fn schema_id() -> Cow<'static, str> {
                Cow::Borrowed(stringify!(std::num::$type))
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
