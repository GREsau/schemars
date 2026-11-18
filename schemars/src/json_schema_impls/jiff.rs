use crate::{json_schema, JsonSchema, Schema, SchemaGenerator};
use alloc::borrow::Cow;

macro_rules! formatted_string_impl {
    ($ty:ident, $format:literal) => {
        impl JsonSchema for $ty {
            inline_schema!();

            fn schema_name() -> Cow<'static, str> {
                stringify!($ty).into()
            }

            fn schema_id() -> Cow<'static, str> {
                concat!("jiff::", stringify!($ty)).into()
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

macro_rules! jiff_impls {
    ($module:ident, $jiff:ident, $zoned:ident) => {
        mod $module {
            use super::*;
            use $jiff::civil::{Date, DateTime, Time};
            use $jiff::{$zoned, SignedDuration, Span, Timestamp};

            formatted_string_impl!(SignedDuration, "duration");
            formatted_string_impl!(Span, "duration");
            formatted_string_impl!(Timestamp, "date-time");
            formatted_string_impl!($zoned, "zoned-date-time");
            formatted_string_impl!(Date, "date");
            formatted_string_impl!(Time, "partial-time");
            formatted_string_impl!(DateTime, "partial-date-time");
        }
    };
}

#[cfg(feature = "jiff02")]
jiff_impls!(jiff02_impls, jiff02, Zoned);

#[cfg(feature = "jiff1")]
jiff_impls!(jiff1_impls, jiff1, ZonedDateTime);
