use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use semver::Version;

impl JsonSchema for Version {
    no_ref_schema!();

    fn schema_name() -> String {
        "Version".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            string: Some(Box::new(StringValidation {
                pattern: Some(r"^\d+\.\d+\.\d+([\-\+].+)?$".to_owned()),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}
