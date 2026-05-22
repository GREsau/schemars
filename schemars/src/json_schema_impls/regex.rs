use crate::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use alloc::borrow::Cow;
use regex::Regex;

impl JsonSchema for Regex {
    inline_schema!();

    fn schema_name() -> Cow<'static, str> {
        "Regex".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "regex::Regex".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "format": "regex",
        })
    }
}
