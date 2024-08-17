use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use alloc::borrow::Cow;
use core::time::Duration;
use std::time::SystemTime;

impl JsonSchema for Duration {
    fn schema_name() -> Cow<'static, str> {
        "Duration".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "std::time::Duration".into()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
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
    fn schema_name() -> Cow<'static, str> {
        "SystemTime".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "std::time::SystemTime".into()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "object",
            "required": ["secs_since_epoch", "nanos_since_epoch"],
            "properties": {
                "secs_since_epoch": u64::json_schema(gen),
                "nanos_since_epoch": u32::json_schema(gen),
            }
        })
    }
}
