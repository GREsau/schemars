use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use semver::Version;

impl JsonSchema for Version {
    no_ref_schema!();

    fn schema_name() -> String {
        "SemVer".to_string()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            string: Some(Box::new(StringValidation {
                pattern: Some("^(?:0|[1-9]\\d*)\\.(?:0|[1-9]\\d*)\\.(?:0|[1-9]\\d*)$".to_string()),
                min_length: Some(5),
                max_length: Some(14),
            })),
            ..Default::default()
        }
        .into()
    }
}
