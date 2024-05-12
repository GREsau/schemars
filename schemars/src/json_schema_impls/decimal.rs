use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use std::borrow::Cow;

macro_rules! decimal_impl {
    ($type:ty) => {
        impl JsonSchema for $type {
            no_ref_schema!();

            fn schema_name() -> String {
                "Decimal".to_owned()
            }

            fn schema_id() -> Cow<'static, str> {
                Cow::Borrowed("Decimal")
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({
                    "type": "string",
                    "pattern": r"^-?[0-9]+(\.[0-9]+)?$",
                })
            }
        }
    };
}

#[cfg(feature = "rust_decimal1")]
decimal_impl!(rust_decimal1::Decimal);
#[cfg(feature = "bigdecimal04")]
decimal_impl!(bigdecimal04::BigDecimal);
