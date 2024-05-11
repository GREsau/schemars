use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use semver1::Version;
use std::borrow::Cow;

impl JsonSchema for Version {
    no_ref_schema!();

    fn schema_name() -> String {
        "Version".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("semver::Version")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            // https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
            "pattern": r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$"
        })
    }
}
