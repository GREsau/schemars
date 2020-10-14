use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime, Time};

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

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                SchemaObject {
                    instance_type: Some(InstanceType::String.into()),
                    format: Some($format.to_owned()),
                    ..Default::default()
                }
                .into()
            }
        }
    };
}

formatted_string_impl!(Date, "date");
formatted_string_impl!(Time, "time");
formatted_string_impl!(PrimitiveDateTime, "partial-date-time");
formatted_string_impl!(OffsetDateTime, "date-time");
forward_impl!(Duration => std::time::Duration);
