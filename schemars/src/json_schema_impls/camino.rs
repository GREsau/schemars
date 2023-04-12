use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use camino::{Utf8Path, Utf8PathBuf};

impl JsonSchema for Utf8Path {
    no_ref_schema!();

    fn schema_name() -> String {
        "String".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            ..Default::default()
        }
            .into()
    }
}

impl JsonSchema for Utf8PathBuf {
    no_ref_schema!();

    fn schema_name() -> String {
        "String".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            ..Default::default()
        }
            .into()
    }
}
