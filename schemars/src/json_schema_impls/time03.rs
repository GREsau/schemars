use crate::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use alloc::borrow::Cow;

use time03::{Date, OffsetDateTime, PrimitiveDateTime, Time};

macro_rules! formatted_string_impl {
    ($ty:ident, $format:literal) => {
        formatted_string_impl!($ty, $format, JsonSchema for $ty);
    };
    ($ty:ident, $format:literal, $($desc:tt)+) => {
        impl $($desc)+ {
            inline_schema!();

            fn schema_name() -> Cow<'static, str> {
                stringify!($ty).into()
            }

            fn schema_id() -> Cow<'static, str>  {
                stringify!(time::$ty).into()
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({
                    "type": "string",
                    "format": $format
                })
            }
        }
    };
}

formatted_string_impl!(Date, "date");
formatted_string_impl!(PrimitiveDateTime, "partial-date-time");
formatted_string_impl!(Time, "time");
formatted_string_impl!(OffsetDateTime, "date-time");
