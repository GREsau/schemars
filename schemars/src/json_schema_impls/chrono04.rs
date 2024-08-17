use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use chrono04::prelude::*;
use alloc::borrow::Cow;

impl JsonSchema for Weekday {
    always_inline!();

    fn schema_name() -> Cow<'static, str> {
        "Weekday".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "chrono::Weekday".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "enum": [
                "Mon",
                "Tue",
                "Wed",
                "Thu",
                "Fri",
                "Sat",
                "Sun",
            ]
        })
    }
}

macro_rules! formatted_string_impl {
    ($ty:ident, $format:literal) => {
        formatted_string_impl!($ty, $format, JsonSchema for $ty);
    };
    ($ty:ident, $format:literal, $($desc:tt)+) => {
        impl $($desc)+ {
            always_inline!();

            fn schema_name() -> Cow<'static, str> {
                stringify!($ty).into()
            }

            fn schema_id() -> Cow<'static, str>  {
                stringify!(chrono::$ty).into()
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

formatted_string_impl!(NaiveDate, "date");
formatted_string_impl!(NaiveDateTime, "partial-date-time");
formatted_string_impl!(NaiveTime, "partial-date-time");
formatted_string_impl!(DateTime, "date-time", <Tz: TimeZone> JsonSchema for DateTime<Tz>);
