use bigdecimal::BigDecimal;

use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;

impl JsonSchema for BigDecimal {
    no_ref_schema!();

    fn schema_name() -> String {
        "BigDecimal".to_string()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            string: Some(Box::new(StringValidation {
                pattern: Some("^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)$".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}
