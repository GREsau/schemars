use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use ordered_float::OrderedFloat;

impl JsonSchema for OrderedFloat<f32> {
    no_ref_schema!();

    fn schema_name() -> String {
        "float".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Number.into()),
            format: Some("float".to_owned()),
            ..Default::default()
        }
        .into()
    }
}

impl JsonSchema for OrderedFloat<f64> {
    no_ref_schema!();

    fn schema_name() -> String {
        "double".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Number.into()),
            format: Some("double".to_owned()),
            ..Default::default()
        }
        .into()
    }
}
