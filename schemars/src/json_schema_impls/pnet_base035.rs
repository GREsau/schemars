use crate::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use alloc::borrow::Cow;
use pnet_base035::MacAddr;

impl JsonSchema for MacAddr {
    inline_schema!();

    fn schema_name() -> Cow<'static, str> {
        "MacAddr".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "pnet_base::MacAddr".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": "^([0-9A-Fa-f]{2}[:]){5}([0-9A-Fa-f]{2})$"
        })
    }
}
