use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use rocket::fs::TempFile;
impl JsonSchema for TempFile<'_> {
    no_ref_schema!();

    fn schema_name() -> String {
        "TempFile".to_string()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some("Contains the uploaded file".to_string()),
                examples: vec![serde_json::to_value(
                    "some binary string".parse::<String>().unwrap(),
                )]
                .into_iter()
                .flatten()
                .collect(),
                ..Default::default()
            })),
            instance_type: Some(InstanceType::String.into()),
            format: Some("binary".to_string()),
            ..Default::default()
        }
        .into()
    }
}
