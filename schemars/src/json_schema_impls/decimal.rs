use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use std::borrow::Cow;

macro_rules! decimal_impl {
    ($type:ty) => {
        impl JsonSchema for $type {
            no_ref_schema!();

            fn schema_name() -> String {
                "Decimal".to_owned()
            }

            fn schema_id() -> Cow<'static, str> {
                Cow::Borrowed($name)
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                SchemaObject {
                    instance_type: Some(InstanceType::String.into()),
                    string: Some(Box::new(StringValidation {
                        pattern: Some(r"^-?[0-9]+(\.[0-9]+)?$".to_owned()),
                        ..Default::default()
                    })),
                    ..Default::default()
                }
                .into()
            }
        }
    };
}

#[cfg(feature = "rust_decimal")]
decimal_impl!(rust_decimal::Decimal);
#[cfg(feature = "bigdecimal")]
decimal_impl!(bigdecimal::BigDecimal);
