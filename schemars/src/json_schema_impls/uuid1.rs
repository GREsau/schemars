use crate::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use alloc::borrow::Cow;
use uuid1::Uuid;

impl JsonSchema for Uuid {
    inline_schema!();

    fn schema_name() -> Cow<'static, str> {
        "Uuid".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "uuid::Uuid".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "format": "uuid",
        })
    }
}

impl JsonSchema for uuid1::fmt::Simple {
    fn schema_name() -> Cow<'static, str> {
        "Simple".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "uuid::fmt::Simple".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": r"^[0-9a-f]{32}$"
        })
    }
}

impl JsonSchema for uuid1::fmt::Braced {
    fn schema_name() -> Cow<'static, str> {
        "Braced".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "uuid::fmt::Braced".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": r"^\{[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\}$",
        })
    }
}

impl JsonSchema for uuid1::fmt::Hyphenated {
    fn schema_name() -> Cow<'static, str> {
        "Hyphenated".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "uuid::fmt::Hyphenated".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "format": "uuid",
        })
    }
}

impl JsonSchema for uuid1::fmt::Urn {
    fn schema_name() -> Cow<'static, str> {
        "Urn".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "uuid::fmt::Urn".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": r"^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        })
    }
}
