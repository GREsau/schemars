use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use std::borrow::Cow;
use url2::Url;

impl JsonSchema for Url {
    no_ref_schema!();

    fn schema_name() -> String {
        "Url".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("url::Url")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "format": "uri",
        })
    }
}
