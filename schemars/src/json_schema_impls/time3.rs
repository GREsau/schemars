use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use time3::OffsetDateTime;

impl JsonSchema for OffsetDateTime {
    no_ref_schema!();

    fn schema_name() -> String {
        "DateTime".to_string()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("date-time".to_string()),
            ..Default::default()
        }
        .into()
    }
}
