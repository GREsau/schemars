use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;

#[cfg(feature = "rust_decimal")]
impl JsonSchema for rust_decimal::Decimal {
    no_ref_schema!();

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Number.into()),
            ..Default::default()
        }
        .into()
    }
}

#[cfg(feature = "bigdecimal")]
impl JsonSchema for bigdecimal::BigDecimal {
    no_ref_schema!();

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Number.into()),
            ..Default::default()
        }
        .into()
    }
}
