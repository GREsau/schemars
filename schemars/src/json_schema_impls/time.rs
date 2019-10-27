use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use std::time::{Duration, SystemTime};

impl JsonSchema for Duration {
    fn schema_name() -> String {
        "Duration".to_owned()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = SchemaObject::default();
        schema.instance_type = Some(InstanceType::Object.into());
        let properties = &mut schema.object().properties;
        properties.insert("secs".to_owned(), <u64>::json_schema(gen));
        properties.insert("nanos".to_owned(), <u32>::json_schema(gen));
        schema.into()
    }
}

impl JsonSchema for SystemTime {
    fn schema_name() -> String {
        "SystemTime".to_owned()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = SchemaObject::default();
        schema.instance_type = Some(InstanceType::Object.into());
        let properties = &mut schema.object().properties;
        properties.insert("secs_since_epoch".to_owned(), <u64>::json_schema(gen));
        properties.insert("nanos_since_epoch".to_owned(), <u32>::json_schema(gen));
        schema.into()
    }
}
