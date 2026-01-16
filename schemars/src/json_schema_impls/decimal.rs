use crate::_alloc_prelude::*;
use crate::{JsonSchema, Schema, SchemaGenerator};
use alloc::borrow::Cow;

#[cfg(any(
    feature = "bigdecimal04",
    all(feature = "rust_decimal1", not(feature = "rust_decimal1_serde-float"))
))]
use crate::generate::Contract;

#[cfg(any(
    feature = "bigdecimal04",
    all(feature = "rust_decimal1", not(feature = "rust_decimal1_serde-float"))
))]
use serde_json::Value;

#[cfg_attr(
    all(feature = "rust_decimal1", not(feature = "bigdecimal04")),
    allow(unused_macros)
)]
macro_rules! decimal_impl {
    ($type:ty) => {
        impl JsonSchema for $type {
            inline_schema!();

            fn schema_name() -> Cow<'static, str> {
                "Decimal".into()
            }

            fn json_schema(generator: &mut SchemaGenerator) -> Schema {
                let (ty, pattern) = match generator.contract() {
                    Contract::Deserialize => (
                        Value::Array(vec!["string".into(), "number".into()]),
                        r"^-?\d+(\.\d+)?([eE]\d+)?$".into(),
                    ),
                    Contract::Serialize => ("string".into(), r"^-?\d+(\.\d+)?$".into()),
                };

                let mut result = Schema::default();
                result.insert("type".to_owned(), ty);
                result.insert("pattern".to_owned(), pattern);
                result
            }
        }
    };
}

#[cfg(feature = "rust_decimal1")]
impl JsonSchema for rust_decimal1::Decimal {
    inline_schema!();

    fn schema_name() -> Cow<'static, str> {
        "Decimal".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        #[cfg(feature = "rust_decimal1_serde-float")]
        {
            let _generator = generator;
            let ty = "number";
            let mut result = Schema::default();
            result.insert("type".to_owned(), ty.into());
            return result;
        }

        #[cfg(not(feature = "rust_decimal1_serde-float"))]
        {
            let (ty, pattern) = match generator.contract() {
                Contract::Deserialize => (
                    Value::Array(vec!["string".into(), "number".into()]),
                    r"^-?\d+(\.\d+)?([eE]\d+)?$".into(),
                ),
                Contract::Serialize => ("string".into(), r"^-?\d+(\.\d+)?$".into()),
            };

            let mut result = Schema::default();
            result.insert("type".to_owned(), ty);
            result.insert("pattern".to_owned(), pattern);
            result
        }
    }
}

#[cfg(feature = "bigdecimal04")]
decimal_impl!(bigdecimal04::BigDecimal);
