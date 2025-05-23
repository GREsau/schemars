use crate::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use alloc::borrow::Cow;
use bytesize2::ByteSize;

impl JsonSchema for ByteSize {
    fn schema_name() -> Cow<'static, str> {
        "ByteSize".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "bytesize::ByteSize".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": r"^(\d+(\.\d+)?)\s*((?i)[kmgtp]i?b?)?$"
        })
    }
}
