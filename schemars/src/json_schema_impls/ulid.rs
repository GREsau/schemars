use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use ulid::Ulid;

impl JsonSchema for Ulid {
    no_ref_schema!();

    fn schema_name() -> String {
        "Ulid".to_string()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("ulid".to_string()),
            ..Default::default()
        }
        .into()
    }
}
