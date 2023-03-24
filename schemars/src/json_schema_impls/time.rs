use crate::gen::SchemaGenerator;
use crate::schema::{InstanceType, Schema, SchemaObject};
use crate::JsonSchema;
use time::OffsetDateTime;

impl JsonSchema for OffsetDateTime {
    fn is_referenceable() -> bool {
        false
    }

    fn schema_name() -> String {
        "DateTime".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("date-time".into()),
            ..Default::default()
        }
        .into()
    }
}
