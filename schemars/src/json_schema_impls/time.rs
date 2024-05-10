use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use std::borrow::Cow;
use std::time::{Duration, SystemTime};

impl JsonSchema for Duration {
    fn schema_name() -> String {
        "Duration".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("std::time::Duration")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        crate::json_schema!({
            "type": "object",
            "required": ["secs", "nanos"],
            "properties": {
                "secs": u64::json_schema(gen),
                "nanos": u32::json_schema(gen),
            }
        })
    }
}

impl JsonSchema for SystemTime {
    fn schema_name() -> String {
        "SystemTime".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("std::time::SystemTime")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        crate::json_schema!({
            "type": "object",
            "required": ["secs_since_epoch", "nanos_since_epoch"],
            "properties": {
                "secs_since_epoch": u64::json_schema(gen),
                "nanos_since_epoch": u32::json_schema(gen),
            }
        })
    }
}
