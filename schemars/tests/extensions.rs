mod util;
use schemars::JsonSchema;
use serde::Serialize;
use util::*;

#[derive(Default, Debug, JsonSchema, Serialize)]
#[schemars(extension = "struct_tag_1", extension = "struct_tag_2")]
pub struct Struct {
    #[schemars(extension = "field_tag_1", extension = "field_tag_2")]
    foo: i32,
    bar: bool,
}

fn struct_tag_1() -> (String, String) {
    ("x-tag-1".into(), "struct-tag-1".into())
}

fn struct_tag_2() -> (String, String) {
    ("x-tag-2".into(), "struct-tag-2".into())
}

fn field_tag_1() -> (String, String) {
    ("x-tag-1".into(), "field-tag-1".into())
}

fn field_tag_2() -> (String, String) { ("x-tag-2".into(), "field-tag-2".into()) }

#[test]
fn extensions() -> TestResult {
    test_default_generated_schema::<Struct>("extensions")
}
