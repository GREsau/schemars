use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use chrono04::prelude::*;
use std::borrow::Cow;

impl JsonSchema for Weekday {
    no_ref_schema!();

    fn schema_name() -> String {
        "Weekday".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("chrono::Weekday")
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
            no_ref_schema!();

            fn schema_name() -> String {
                stringify!($ty).to_owned()
            }

            fn schema_id() -> Cow<'static, str>  {
                Cow::Borrowed(stringify!(chrono::$ty))
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
